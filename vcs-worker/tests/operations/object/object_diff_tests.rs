//! Unit tests for object_diff module
//!
//! These tests verify that verbs with multiple names are counted correctly
//! (as single verbs, not multiple) in ObjectChange tracking.

use moor_vcs_worker::object_diff::{compare_object_definitions_with_meta, ObjectChange};
use moor_compiler::ObjectDefinition;

// Helper function to create a verb definition with multiple names
fn create_verb_def(names: Vec<&str>) -> moor_compiler::ObjVerbDef {
    use moor_compiler::ObjVerbDef;
    use moor_var::{Symbol, Obj, program::ProgramType};
    use moor_common::model::VerbArgsSpec;
    use moor_common::util::BitEnum;

    ObjVerbDef {
        names: names.iter().map(|n| Symbol::mk(n)).collect(),
        argspec: VerbArgsSpec::this_none_this(),
        owner: Obj::mk_id(1),
        flags: BitEnum::new(),
        program: ProgramType::MooR(Default::default()),
    }
}

// Helper function to create a property definition
fn create_prop_def(name: &str, value: moor_var::Var) -> moor_compiler::ObjPropDef {
    use moor_compiler::ObjPropDef;
    use moor_var::{Symbol, Obj};
    use moor_common::model::{PropPerms, PropFlag};
    use moor_common::util::BitEnum;

    ObjPropDef {
        name: Symbol::mk(name),
        perms: PropPerms::new(Obj::mk_id(1), BitEnum::<PropFlag>::new()),
        value: Some(value),
    }
}

// Helper function to create a basic object definition
fn create_object_def() -> ObjectDefinition {
    use moor_var::Obj;
    use moor_common::util::BitEnum;

    ObjectDefinition {
        oid: Obj::mk_id(1),
        parent: Obj::mk_id(0),
        location: Obj::mk_id(0),
        owner: Obj::mk_id(1),
        name: "test".to_string(),
        flags: BitEnum::new(),
        verbs: vec![],
        property_definitions: vec![],
        property_overrides: vec![],
    }
}

#[test]
fn test_verb_with_multiple_names_added() {
    // Test that a verb with multiple names is counted as ONE verb added, not multiple
    let baseline = create_object_def();
    
    let mut local = create_object_def();
    // Add a verb with 3 names: "@foo", "foo", "bar"
    local.verbs.push(create_verb_def(vec!["@foo", "foo", "bar"]));

    let mut object_change = ObjectChange::new("test".to_string());
    compare_object_definitions_with_meta(&baseline, &local, &mut object_change, None, None, None, None);

    // Should only count as ONE verb added
    assert_eq!(object_change.verbs_added.len(), 1, 
        "Expected 1 verb added, got {}: {:?}", 
        object_change.verbs_added.len(), 
        object_change.verbs_added);
    
    // Should be empty
    assert_eq!(object_change.verbs_modified.len(), 0);
    assert_eq!(object_change.verbs_deleted.len(), 0);
}

#[test]
fn test_verb_with_multiple_names_deleted() {
    // Test that a verb with multiple names is counted as ONE verb deleted
    let mut baseline = create_object_def();
    baseline.verbs.push(create_verb_def(vec!["@foo", "foo", "bar"]));
    
    let local = create_object_def();

    let mut object_change = ObjectChange::new("test".to_string());
    compare_object_definitions_with_meta(&baseline, &local, &mut object_change, None, None, None, None);

    // Should only count as ONE verb deleted
    assert_eq!(object_change.verbs_deleted.len(), 1,
        "Expected 1 verb deleted, got {}: {:?}", 
        object_change.verbs_deleted.len(), 
        object_change.verbs_deleted);
    
    assert_eq!(object_change.verbs_added.len(), 0);
    assert_eq!(object_change.verbs_modified.len(), 0);
}

#[test]
fn test_verb_with_multiple_names_modified() {
    // Test that a verb with multiple names is counted as ONE verb modified
    let mut baseline = create_object_def();
    baseline.verbs.push(create_verb_def(vec!["@foo", "foo", "bar"]));
    
    let mut local = create_object_def();
    let mut modified_verb = create_verb_def(vec!["@foo", "foo", "bar"]);
    // Change the owner to make it different
    modified_verb.owner = moor_var::Obj::mk_id(2);
    local.verbs.push(modified_verb);

    let mut object_change = ObjectChange::new("test".to_string());
    compare_object_definitions_with_meta(&baseline, &local, &mut object_change, None, None, None, None);

    // Should only count as ONE verb modified
    assert_eq!(object_change.verbs_modified.len(), 1,
        "Expected 1 verb modified, got {}: {:?}", 
        object_change.verbs_modified.len(), 
        object_change.verbs_modified);
    
    assert_eq!(object_change.verbs_added.len(), 0);
    assert_eq!(object_change.verbs_deleted.len(), 0);
}

#[test]
fn test_multiple_verbs_with_multiple_names() {
    // Test multiple verbs, each with multiple names
    let baseline = create_object_def();
    
    let mut local = create_object_def();
    local.verbs.push(create_verb_def(vec!["@foo", "foo", "bar"]));
    local.verbs.push(create_verb_def(vec!["@baz", "baz"]));
    local.verbs.push(create_verb_def(vec!["single"]));

    let mut object_change = ObjectChange::new("test".to_string());
    compare_object_definitions_with_meta(&baseline, &local, &mut object_change, None, None, None, None);

    // Should count as THREE verbs added (not 6)
    assert_eq!(object_change.verbs_added.len(), 3,
        "Expected 3 verbs added, got {}: {:?}", 
        object_change.verbs_added.len(), 
        object_change.verbs_added);
}

#[test]
fn test_mixed_verb_operations() {
    // Test a complex scenario with added, modified, and deleted verbs
    let mut baseline = create_object_def();
    baseline.verbs.push(create_verb_def(vec!["@keep", "keep"]));
    baseline.verbs.push(create_verb_def(vec!["@delete", "delete", "del"]));
    
    let mut local = create_object_def();
    // Keep one verb but modify it
    let mut modified = create_verb_def(vec!["@keep", "keep"]);
    modified.owner = moor_var::Obj::mk_id(2);
    local.verbs.push(modified);
    // Add a new verb
    local.verbs.push(create_verb_def(vec!["@new", "new", "n"]));

    let mut object_change = ObjectChange::new("test".to_string());
    compare_object_definitions_with_meta(&baseline, &local, &mut object_change, None, None, None, None);

    assert_eq!(object_change.verbs_modified.len(), 1,
        "Expected 1 verb modified, got {}", object_change.verbs_modified.len());
    assert_eq!(object_change.verbs_added.len(), 1,
        "Expected 1 verb added, got {}", object_change.verbs_added.len());
    assert_eq!(object_change.verbs_deleted.len(), 1,
        "Expected 1 verb deleted, got {}", object_change.verbs_deleted.len());
}

#[test]
fn test_properties_are_not_affected() {
    // Verify that properties work correctly (they should, since they have single names)
    let baseline = create_object_def();
    
    let mut local = create_object_def();
    local.property_definitions.push(create_prop_def("prop1", moor_var::v_int(42)));
    local.property_definitions.push(create_prop_def("prop2", moor_var::v_str("test")));

    let mut object_change = ObjectChange::new("test".to_string());
    compare_object_definitions_with_meta(&baseline, &local, &mut object_change, None, None, None, None);

    // Should count as TWO properties added
    assert_eq!(object_change.props_added.len(), 2,
        "Expected 2 properties added, got {}: {:?}", 
        object_change.props_added.len(), 
        object_change.props_added);
}

