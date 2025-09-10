object #48
  name: "Generic Recycling Pool"
  location: #-1
  owner: #36
  parent: #1
  readable: true

  override "aliases" = {"Recycling", "Pool", "Recycling Pool"};

  property "owners" (owner: #2, flags: "r") = {};

  property "allocation_range" (owner: #2, flags: "r") = {0, 0};

  property "orphans" (owner: #2, flags: "r") = {};

  property "max_object_id" (owner: #2, flags: "r") = {};

  verb "create" (this none this) owner: #2 flags: "rxd"
    ":create(OBJ of_parent) => Created OBJ";
    "  Used to create objects and bring them into the game!";
    {of_parent} = args;
    if (this.max_object_id >= this.allocation_range[2] && !this.orphans)
      raise (E_QUOTA, "All objects in pool have been allocated.");
    elseif (!$ou:isoneof(caller, from_pool.owners))
      raise (E_PERM, tostr("Caller ", $su:nn(caller), " does not have sufficient permissions to create from ", $su:nn(from_pool), "."));
    endif
    if (this.orphans)
      {orphan_id, orphans} = $lu:pop(this.orphans);
      this.orphans = orphans;
      new_obj = create_at(orphan_id, of_parent);
      new_obj:initialize();
      return new_obj;
    endif
    this.max_object_id = this.max_object_id + 1;
    new_obj = create_at(this.max_object_id, of_parent);
    new_obj:initialize();
    return new_obj;
  endverb
endobject
