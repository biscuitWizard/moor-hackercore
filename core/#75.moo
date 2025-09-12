object #75
  name: "Builder Feature"
  parent: #74
  location: #83
  owner: #36
  readable: true

  override "aliases" = {"Builder", "Feature", "Builder Feature"};

  verb "@children @kids" (any none any) owner: #36 flags: "rxd"
    "Usage:  @tree <object> [-d] [search objects by name]";
    "Shows an object hierarchy beginning with the given object. All fertile children";
    "and children with children are included in the tree.  The output includes the";
    "object number, the object name, and the total number of children the object has.";
    "With option '-d', number of verbs and properties defined.";
    if (!dobjstr)
      return player:tell("@children <object> [-d] [search objects by name]");
    endif
    parameters = dobjstr;
    details = 0;
    if (index(dobjstr, "-d"))
      details = 1;
      parameters = $string_utils:subst(parameters, {{"-d", ""}});
    endif
    parts = $string_utils:explode(parameters);
    root = player:match(parts[1]);
    search_text = "";
    if (length(parts) > 1)
      search_text = $string_utils:from_list(parts[2..$]);
    endif
    if (!valid(root))
      return player:tell("Invalid object reference: ", dobjstr);
    endif
    player:tell("Fertile kids of ", root.name, ": (this may take a while)");
    this:do_tree(root, "", search_text, details, $false);
    player:tell("----- End of @tree listing for ", $string_utils:nn(root), details ? " [kids, verbs and properties]" | "");
  endverb

  verb "@tree" (any none any) owner: #36 flags: "rxd"
    "Usage:  @tree <object> [-d] [search objects by name]";
    "Shows an object hierarchy beginning with the given object. All fertile children";
    "and children with children are included in the tree.  The output includes the";
    "object number, the object name, and the total number of children the object has.";
    "With option '-d', number of verbs and properties defined.";
    if (!dobjstr)
      return player:tell("@tree <object> [-d] [search objects by name]");
    endif
    parameters = dobjstr;
    details = 0;
    if (index(dobjstr, "-d"))
      details = 1;
      parameters = $string_utils:subst(parameters, {{"-d", ""}});
    endif
    parts = $string_utils:explode(parameters);
    root = player:match(parts[1]);
    search_text = "";
    if (length(parts) > 1)
      search_text = $string_utils:from_list(parts[2..$]);
    endif
    if (!valid(root))
      return player:tell("Invalid object reference: ", dobjstr);
    endif
    player:tell("Fertile kids of ", root.name, ": (this may take a while)");
    this:do_tree(root, "", search_text, details);
    player:tell("----- End of @tree listing for ", $string_utils:nn(root), details ? " [kids, verbs and properties]" | "");
  endverb

  verb "do_tree" (this none this) owner: #36 flags: "rxd"
    ":do_tree(OBJ root, INT indent, STR search_text, [INT details, BOOL require_fertile = 1, STR color = \"normal\"])";
    "traverse fertile kids of given root object";
    {root, indent, search_text, ?details = 0, ?require_fertile = 1, ?color = "normal"} = args;
    kids = children(root);
    limit = 70;
    if (length(search_text) > 0 ? index(root.name, search_text) | 1)
      line = tostr($string_utils:space(indent, "  | | | | | | | | |"), "|-", $string_utils:nn(root));
      len = length(line);
      if (len > limit)
        line = line[1..limit];
      elseif (len < limit)
        line = $su:left(line, limit);
      endif
      line = tostr(line, $string_utils:right(length(kids), 4));
      if (details)
        v = verbs(root);
        p = properties(root);
        line = tostr(line, " ", $string_utils:right(v ? length(v) | " ", 3), " ", $string_utils:right(p ? length(p) | " ", 4));
      endif
      player:tell($ansi:(color)(line));
      color = color == "normal" ? "bright" | "normal";
    endif
    indent = indent + "  ";
    hadfertile = 0;
    for kid in (kids)
      if (!require_fertile || kid.f || children(kid))
        {color, spaced} = this:do_tree(kid, indent, search_text, details, require_fertile, color);
        hadfertile = 1;
      endif
      if (kid == kids[$] && hadfertile && !spaced)
        if (callers()[1][2] == "do_tree" && children(parent(root))[$] != root)
          player:tell($ansi:(color)($string_utils:space(indent, "  | | | | | | | | |")));
          color = color == "normal" ? "bright" | "normal";
        endif
      endif
    endfor
    return {color, hadfertile};
  endverb

  verb "@unlock" (any none none) owner: #2 flags: "rd"
    set_task_perms(player);
    dobj = player:match(dobjstr);
    if ($command_utils:object_match_failed(dobj, dobjstr))
      return;
    endif
    try
      dobj.key = 0;
      player:notify(tostr("Unlocked ", dobj.name, "."));
    except error (ANY)
      player:notify(error[2]);
    endtry
  endverb

  verb "@lock" (any with any) owner: #2 flags: "rd"
    set_task_perms(player);
    dobj = player:match(dobjstr);
    if ($command_utils:object_match_failed(dobj, dobjstr))
      return;
    endif
    key = $lock_utils:parse_keyexp(iobjstr, player);
    if (typeof(key) == STR)
      player:notify("That key expression is malformed:");
      player:notify(tostr("  ", key));
    else
      try
        dobj.key = key;
        player:notify(tostr("Locked ", dobj.name, " to this key:"));
        player:notify(tostr("  ", $lock_utils:unparse_key(key)));
      except error (ANY)
        player:notify(error[2]);
      endtry
    endif
  endverb

  verb "@newmess*age" (any any any) owner: #2 flags: "rd"
    "Usage:  @newmessage <message-name> [<message>] [on <object>]";
    "Add a message property to an object (default is player), and optionally";
    "set its value.  For use by non-programmers, who aren't allowed to add";
    "properties generally.";
    "To undo the effects of this, use @unmessage.";
    set_task_perms(player);
    dobjwords = $string_utils:words(dobjstr);
    if (!dobjwords)
      player:notify(tostr("Usage:  ", verb, " <message-name> [<message>] [on <object>]"));
      return;
    endif
    object = valid(iobj) ? iobj | player;
    name = this:_messagify(dobjwords[1]);
    value = dobjstr[length(dobjwords[1]) + 2..$];
    nickname = "@" + name[1..$ - 4];
    e = `add_property(object, name, value, {player, "rc"}) ! ANY';
    if (typeof(e) != ERR)
      player:notify(tostr(nickname, " on ", object.name, " is now \"", object.(name), "\"."));
    elseif (e != E_INVARG)
      player:notify(tostr(e));
    elseif ($object_utils:has_property(object, name))
      "object already has property";
      player:notify(tostr(object.name, " already has a ", nickname, " message."));
    else
      player:notify(tostr("Unable to add ", nickname, " message to ", object.name, ": ", e));
    endif
  endverb

  verb "@unmess*age" (any any any) owner: #2 flags: "rd"
    "Usage:  @unmessage <message-name> [from <object>]";
    "Remove a message property from an object (default is player).";
    set_task_perms(player);
    if (!dobjstr || length($string_utils:words(dobjstr)) > 1)
      player:notify(tostr("Usage:  ", verb, " <message-name> [from <object>]"));
      return;
    endif
    object = valid(iobj) ? iobj | player;
    name = this:_messagify(dobjstr);
    nickname = "@" + name[1..$ - 4];
    try
      delete_property(object, name);
      player:notify(tostr(nickname, " message removed from ", object.name, "."));
    except (E_PROPNF)
      player:notify(tostr("No ", nickname, " message found on ", object.name, "."));
    except error (ANY)
      player:notify(error[2]);
    endtry
  endverb

  verb "_messagify" (this none this) owner: #2 flags: "rxd"
    "Given any of several formats people are likely to use for a @message";
    "property, return the canonical form (\"foobar_msg\").";
    name = args[1];
    if (name[1] == "@")
      name = name[2..$];
    endif
    if (length(name) < 4 || name[$ - 3..$] != "_msg")
      name = name + "_msg";
    endif
    return name;
  endverb

  verb "@par*ents" (any none none) owner: #36 flags: "rd"
    "'@parents <thing>' - List <thing> and its ancestors, all the way back to the Root Class (#1).";
    if (!dobjstr)
      player:notify(tostr("Usage:  ", verb, " <object>"));
      return;
    endif
    set_task_perms(player);
    o = player:match(dobjstr);
    if (!$command_utils:object_match_failed(o, dobjstr))
      object_parents = $string_utils:names_of_indented($list_utils:remove_duplicates({o, @parents(o), @$object_utils:ancestors(o)}));
      for x in (object_parents)
        player:tell(x);
      endfor
    endif
  endverb

  verb "@location*s" (any none none) owner: #2 flags: "rd"
    "@locations <thing> - List <thing> and its containers, all the way back to the outermost one.";
    set_task_perms(player);
    if (!dobjstr)
      what = player;
    elseif (!valid(what = player:match(dobjstr)) && !valid(what = $string_utils:match_player(dobjstr)))
      $command_utils:object_match_failed(dobj, dobjstr);
      return;
    endif
    player:notify($string_utils:names_of({what, @$object_utils:locations(what)}));
  endverb

  verb "@chparent" (any at any) owner: #36 flags: "rd"
    set_task_perms(player);
    if ($command_utils:object_match_failed(object = player:match(dobjstr), dobjstr))
      "...bogus object...";
    elseif ($command_utils:object_match_failed(parent = player:match(iobjstr), iobjstr))
      "...bogus new parent...";
    elseif (this != player && !$object_utils:isa(player, $player))
      "...They chparented to #1 and want to chparent back to $prog.  Probably for some nefarious purpose...";
      player:notify("You don't seem to already be a valid player class.  Perhaps chparenting away from the $player hierarchy was not such a good idea.  Permission denied.");
    elseif (is_player(object) && !$object_utils:isa(parent, $player))
      player:notify(tostr(object, " is a player and ", parent, " is not a player class."));
      player:notify("You really *don't* want to do this.  Trust me.");
    else
      if ($object_utils:isa(object, $mail_recipient))
        if (!$command_utils:yes_or_no("Chparenting a mailing list is usually a really bad idea.  Do you really want to do it?  (If you don't know why we're asking this question, please say 'no'.)"))
          return player:tell("Aborted.");
        endif
      endif
      try
        result = player:_chparent(object, parent);
        player:notify("Parent changed.");
      except (E_INVARG)
        if (valid(object) && valid(parent))
          player:notify(tostr("Some property existing on ", parent, " is defined on ", object, " or one of its descendants."));
          player:notify(tostr("Try @check-chparent ", dobjstr, " to ", iobjstr));
        else
          player:notify("Either that is not a valid object or not a valid parent");
        endif
      except (E_PERM)
        player:notify("Either you don't own the object, don't own the parent, or the parent is not fertile.");
      except (E_RECMOVE)
        player:notify("That parent object is a descendant of the object!");
      endtry
    endif
  endverb

  verb "@grep*all @egrep*all" (any any any) owner: #36 flags: "rd"
    "Copied from Generic Agent (#58):@grep [verb author FishPot (#2)] at Thu Jan  5 08:47:14 2023 UTC";
    if (!this.programmer)
      return this:tell("This can only be used by programmers.");
    endif
    set_task_perms(player);
    if (prepstr == "in")
      pattern = dobjstr;
      objlist = player:eval_cmd_string(iobjstr, 0);
      if (!objlist[1])
        player:notify(tostr("Had trouble reading `", iobjstr, "':  "));
        player:notify_lines(@objlist[2]);
        return;
      elseif (typeof(objlist[2]) == OBJ)
        objlist = {objlist[2..2]};
      elseif (typeof(objlist[2]) != LIST)
        player:notify(tostr("Value of `", iobjstr, "' is not an object or list:  ", toliteral(objlist[2])));
        return;
      else
        objlist = objlist[2..2];
      endif
    elseif (prepstr == "from" && (player.wizard && (n = toint(toobj(iobjstr)))))
      pattern = dobjstr;
      objlist = {n};
    elseif (args)
      pattern = argstr;
      objlist = {};
    else
      player:notify(tostr("Usage:  ", verb, " <pattern> ", player.wizard ? "[in {<objectlist>} | from <number>]" | "in {<objectlist>}"));
      return;
    endif
    start = ftime();
    player:notify(tostr("Searching for verbs ", @prepstr ? {prepstr, " ", iobjstr, " "} | {}, verb == "@egrep" ? "matching the pattern " | "containing the string ", toliteral(pattern), " ..."));
    player:notify("");
    egrep = verb[2] == "e";
    all = index(verb, "a");
    $code_utils:(all ? egrep ? "find_verb_lines_matching" | "find_verb_lines_containing" | (egrep ? "find_verbs_matching" | "find_verbs_containing"))(pattern, @objlist);
    player:notify("Grep completed in: ", ftime() - start, "s");
  endverb

  verb "@check-p*roperty" (any none none) owner: #2 flags: "rd"
    "@check-prop object.property";
    "  checks for descendents defining the given property.";
    set_task_perms(player);
    if (!(spec = $code_utils:parse_propref(dobjstr)))
      player:notify(tostr("Usage:  ", verb, " <object>.<prop-name>"));
    elseif ($command_utils:object_match_failed(object = player:match(spec[1]), spec[1]))
      "...bogus object...";
    elseif (!($perm_utils:controls(player, object) || object.w))
      player:notify("You can't create a property on that object anyway.");
    elseif ($object_utils:has_property(object, prop = spec[2]))
      player:notify("That object already has that property.");
    elseif (olist = $object_utils:descendants_with_property(object, prop))
      player:notify("The following descendents have this property defined:");
      player:notify("  " + $string_utils:from_list(olist, " "));
    else
      player:notify("No property name conflicts found.");
    endif
  endverb
endobject