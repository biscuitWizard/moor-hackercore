object #77
  name: "Builder Options"
  parent: #68
  location: #-1
  owner: #36
  readable: true
  override "aliases" = {"Builder Options"};

  override "description" = {"Option package for $builder commands.  See `help @build-options'."};

  override "names" = {"dig_room", "dig_exit", "create_flags", "bi_create", "audit_bytes", "audit_float"};

  override "_namelist" = "!dig_room!dig_exit!create_flags!bi_create!audit_bytes!audit_float!";

  override "extras" = {};

  override "namewidth" = 25;

  property "show_bi_create" (owner: #36, flags: "rc") = {"@create/@recycle re-use object numbers.", "@create/@recycle call create()/recycle() directly."};

  property "type_dig_room" (owner: #36, flags: "rc") = {1};

  property "type_dig_exit" (owner: #36, flags: "rc") = {1};

  property "show_audit_bytes" (owner: #2, flags: "r") = {"@audit/@prospectus shows `<1K'", "@audit/@prospectus shows bytes"};

  property "show_audit_float" (owner: #2, flags: "r") = {"@audit/@prospectus shows integer sizes (1K)", "@audit/@prospectus shows floating-point sizes (1.0K)"};

  verb "check_create_flags" (this none this) owner: #36 flags: "rxd"
    value = args[1];
    if (m = match(value, "[^rwf]"))
      return tostr("Unknown object flag:  ", value[m[1]]);
    else
      return {tostr(index(value, "r") ? "r" | "", index(value, "w") ? "w" | "", index(value, "f") ? "f" | "")};
    endif
  endverb

  verb "show_create_flags" (this none this) owner: #36 flags: "rxd"
    if (value = this:get(@args))
      return {value, {tostr("Object flags for @create:  ", value)}};
    else
      return {0, {tostr("@create leaves all object flags reset")}};
    endif
  endverb

  verb "parse_create_flags" (this none this) owner: #36 flags: "rxd"
    raw = args[2];
    if (raw == 1)
      "...+create_flags => create_flags=r";
      return {args[1], "r"};
    elseif (typeof(raw) == STR)
      return args[1..2];
    elseif (typeof(raw) != LIST)
      return "???";
    elseif (length(raw) > 1)
      return tostr("I don't understand \"", $string_utils:from_list(listdelete(raw, 1), " "), "\"");
    else
      return {args[1], raw[1]};
    endif
  endverb

  verb "show_dig_room show_dig_exit" (this none this) owner: #36 flags: "rxd"
    name = args[2];
    what = verb == "show_dig_room" ? "room" | "exit";
    if ((value = this:get(args[1], name)) == 0)
      return {0, {tostr("@dig ", what, "s are children of $", what, ".")}};
    else
      return {value, {tostr("@dig ", what, "s are children of ", value, " (", valid(value) ? value.name | "invalid", ").")}};
    endif
  endverb

  verb "parse_dig_room parse_dig_exit" (this none this) owner: #36 flags: "rxd"
    {oname, raw, data} = args;
    if (typeof(raw) == LIST)
      if (length(raw) > 1)
        return tostr("I don't understand \"", $string_utils:from_list(listdelete(raw, 1), " "), "\".");
      endif
      raw = raw[1];
    endif
    if (typeof(raw) != STR)
      return "You need to give an object id.";
    elseif ($command_utils:object_match_failed(value = player:match(raw), raw))
      return "Option unchanged.";
    endif
    what = verb == "parse_dig_room" ? "room" | "exit";
    generic = #0.(what);
    if (value == generic)
      return {oname, 0};
    else
      if (!$object_utils:isa(value, generic))
        player:tell("Warning: ", value, " is not a descendant of $", what, ".");
      endif
      return {oname, value};
    endif
  endverb

endobject
