object #76
  name: "Programmer Options"
  parent: #68
  location: #-1
  owner: #36
  readable: true
  override "key" = 0;

  override "aliases" = {"Programmer Options"};

  override "description" = {"Option package for $prog commands.  See `help @prog-options'."};

  override "names" = {"list_all_parens", "list_no_numbers", "eval_time", "copy_expert", "verb_args", "verb_perms", "@prop_flags", "list_show_permissions", "rmverb_mail_backup", "//_comments"};

  override "_namelist" = "!list_all_parens!list_no_numbers!list_show_permissions!eval_time!copy_expert!list_numbers!verb_args!@prop_flags!rmverb_mail_backup!//_comments!verb_perms!";

  override "extras" = {"list_numbers"};

  override "namewidth" = 25;

  property "show_eval_time" (owner: #36, flags: "rc") = {"eval does not show ticks/seconds consumed.", "eval shows ticks/seconds consumed."};

  property "show_list_all_parens" (owner: #36, flags: "rc") = {"@list shows only necessary parentheses by default", "@list shows all parentheses by default"};

  property "show_list_no_numbers" (owner: #36, flags: "rc") = {"@list gives line numbers by default", "@list omits line numbers by default"};

  property "show_copy_expert" (owner: #36, flags: "rc") = {"@copy prints warning message.", "@copy omits warning message."};

  property "type_@prop_flags" (owner: #36, flags: "rc") = {2};

  property "show_list_show_permissions" (owner: #36, flags: "rc") = {"@list does not display permissions in header", "@list displays permissions in header"};

  property "show_rmverb_mail_backup" (owner: #36, flags: "rc") = {"@rmverb does not email you a backup", "@rmverb emails you a backup before deleting the verb"};

  property "show_//_comments" (owner: #36, flags: "rc") = {"Comments shown in editors will be MOO-style.", "Comments shown in editors will begin with //"};

  verb "actual" (this none this) owner: #36 flags: "rxd"
    if (i = args[1] in {"list_numbers"})
      return {{{"list_no_numbers"}[i], !args[2]}};
    else
      return {args};
    endif
  endverb

  verb "show" (this none this) owner: #36 flags: "rxd"
    if (o = (name = args[2]) in {"list_numbers"})
      args[2] = {"list_no_numbers"}[o];
      return {@pass(@args), tostr("(", name, " is a synonym for -", args[2], ")")};
    else
      return pass(@args);
    endif
  endverb

  verb "show_verb_args" (this none this) owner: #36 flags: "rxd"
    if (value = this:get(@args))
      return {value, {tostr("Default args for @verb:  ", $string_utils:from_list(value, " "))}};
    else
      return {0, {"Default args for @verb:  none none none"}};
    endif
  endverb

  verb "check_verb_args" (this none this) owner: #36 flags: "rxd"
    value = args[1];
    if (typeof(value) != LIST)
      return "List expected";
    elseif (length(value) != 3)
      return "List of length 3 expected";
    elseif (!(value[1] in {"this", "none", "any"}))
      return tostr("Invalid dobj specification:  ", value[1]);
    elseif (!((p = $code_utils:short_prep(value[2])) || value[2] in {"none", "any"}))
      return tostr("Invalid preposition:  ", value[2]);
    elseif (!(value[3] in {"this", "none", "any"}))
      return tostr("Invalid iobj specification:  ", value[3]);
    else
      if (p)
        value[2] = p;
      endif
      return {value};
    endif
  endverb

  verb "parse_verb_args" (this none this) owner: #36 flags: "rxd"
    {oname, raw, data} = args;
    if (typeof(raw) == STR)
      raw = $string_utils:explode(raw, " ");
    elseif (typeof(raw) == INT)
      return raw ? {oname, {"this", "none", "this"}} | {oname, 0};
    endif
    value = $code_utils:parse_argspec(@raw);
    if (typeof(value) != LIST)
      return tostr(value);
    elseif (value[2])
      return tostr("I don't understand \"", $string_utils:from_list(value[2], " "), "\"");
    else
      value = {@value[1], "none", "none", "none"}[1..3];
      return {oname, value == {"none", "none", "none"} ? 0 | value};
    endif
  endverb

  verb "show_@prop_flags" (this none this) owner: #36 flags: "rxd"
    value = this:get(@args);
    if (value)
      return {value, {tostr("Default permissions for @property=`", value, "'.")}};
    else
      return {0, {"Default permissions for @property=`rc'."}};
    endif
  endverb

  verb "check_@prop_flags" (this none this) owner: #2 flags: "rxd"
  endverb

  verb "parse_@prop_flags" (this none this) owner: #2 flags: "rxd"
    {oname, raw, data} = args;
    if (typeof(raw) != STR)
      return "Must be a string composed of the characters `rwc'.";
    endif
    len = length(raw);
    for x in [1..len]
      if (!(raw[x] in {"r", "w", "c"}))
        return "Must be a string composed of the characters `rwc'.";
      endif
    endfor
    return {oname, raw};
  endverb

  verb "check_verb_perms" (this none this) owner: #2 flags: "rxd"
    value = args[1];
    if (typeof(value) != STR)
      return "Must be a string composed of the characters `RWXD'.";
    elseif ((stripped = $string_utils:subst(value, {{"R", ""}, {"W", ""}, {"X", ""}, {"D", ""}})) != "")
      "I know you can strip_all_but, but we want to report invalid values.";
      return tostr("Invalid permission ", $s("character", length(stripped)), ": ", stripped);
    else
      return {value};
    endif
  endverb

  verb "show_verb_perms" (this none this) owner: #2 flags: "rxd"
    if (value = this:get(@args))
      return {value, {tostr("Default permissions for @verb:  ", value)}};
    else
      return {0, {"Default permissions for @verb:  rd"}};
    endif
  endverb

  verb "parse_verb_perms" (this none this) owner: #2 flags: "rxd"
    {oname, raw, data} = args;
    if (typeof(raw) == STR)
      raw = {raw};
    elseif (typeof(raw) == INT)
      return raw ? {oname, "rxd"} | {oname, 0};
    endif
    value = this:check_verb_perms(raw[1]);
    if (typeof(value) == STR)
      return value;
    endif
    if (value[1] == "")
      value = "RD";
    endif
    return {oname, value[1] == "RD" ? 0 | value[1]};
  endverb

endobject
