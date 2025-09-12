object #4
  name: "Generic Builder"
  parent: #6
  location: #-1
  owner: #2
  readable: true
  override "aliases" = {"Generic Builder"};

  override "description" = "You see a player who should type '@describe me as ...'.";

  override "features" = {#90, #89, #75};

  override "help" = #84;

  override "home" = #62;

  property "build_options" (owner: #2, flags: "rc") = {};

  verb "match" (this none this) owner: #2 flags: "rxd"
    ":match(STR subject) => OBJ match";
    "  This matches <subject> from the perspective of <this>";
    "  If nothing is found then $failed_match is returned";
    "  If multiple things are found then $ambiguous_match is returned";
    {?subject = "", @more} = args;
    if ($recycler:valid(result = pass(@args)))
      return result;
    elseif ($recycler:valid(object = $su:literal_object(subject)))
      return object;
    elseif ($recycler:valid(character = $match_utils:match_character(subject)))
      return character;
    endif
    return $failed_match;
  endverb

  verb "@contents @i*nventory" (any none none) owner: #2 flags: "rd"
    "'@contents <obj> - list the contents of an object, with object numbers.";
    set_task_perms(player);
    if (!dobjstr)
      dobj = player.location;
    else
      dobj = player:match(dobjstr);
    endif
    if ($command_utils:object_match_failed(dobj, dobjstr))
    else
      contents = dobj.contents;
      if (contents)
        player:notify(tostr(dobj:title(), "(", dobj, ") contains:"));
        player:notify(tostr($string_utils:names_of(contents)));
      else
        player:notify(tostr(dobj:title(), "(", dobj, ") contains nothing."));
      endif
    endif
  endverb

  verb "classes_2" (this none this) owner: #2 flags: "rxd"
    {root, indent, members, printed} = args;
    if (root != $nothing)
      if (root in members)
        player:tell(indent, root.name, " (", root, ")");
      else
        player:tell(indent, "<", root.name, " (", root, ")>");
      endif
    endif
    printed = setremove(printed, root);
    if (root != $nothing)
      indent = indent + "  ";
    else
      indent = "";
    endif
    set_task_perms(caller_perms());
    if (root == $nothing)
      "children($nothing) is invalid, so make other arrangements.";
      objects = {};
      for x in (printed)
        if (parent(x) == $nothing)
          objects = {@objects, x};
        endif
      endfor
    else
      objects = $set_utils:intersection(children(root), printed);
    endif
    for c in ($list_utils:sort(2, objects))
      this:classes_2(c, indent, members, printed);
    endfor
  endverb

  verb "@chparent" (any at any) owner: #2 flags: "rd"
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

  verb "@check-chp*arent" (any at any) owner: #2 flags: "rd"
    "Copied from generic programmer (#217):@check-chparent by ur-Rog (#6349) Sun Nov  8 22:13:53 1992 PST";
    "@check-chparent object to newparent";
    "checks for property name conflicts that would make @chparent bomb.";
    set_task_perms(player);
    if (!(dobjstr && iobjstr))
      player:notify(tostr("Usage:  ", verb, " <object> to <newparent>"));
    elseif ($command_utils:object_match_failed(object = player:match(dobjstr), dobjstr))
      "...bogus object...";
    elseif ($command_utils:object_match_failed(parent = player:match(iobjstr), iobjstr))
      "...bogus new parent...";
    elseif (player != this)
      player:notify(tostr(E_PERM));
    elseif (typeof(result = $object_utils:property_conflicts(object, parent)) == ERR)
      player:notify(tostr(result));
    elseif (result)
      su = $string_utils;
      player:notify("");
      player:notify(su:left("Property", 30) + "Also Defined on");
      player:notify(su:left("--------", 30) + "---------------");
      for r in (result)
        player:notify(su:left(tostr(parent, ".", r[1]), 30) + su:from_list(listdelete(r, 1), " "));
      endfor
    else
      player:notify("No property conflicts found.");
    endif
  endverb

  verb "@set*prop" (any at any) owner: #2 flags: "rd"
    "Syntax:  @set <object>.<prop-name> to <value>";
    "";
    "Changes the value of the specified object's property to the given value.";
    "You must have permission to modify the property, either because you own the property or if it is writable.";
    set_task_perms(player);
    if (this != player)
      return player:tell(E_PERM);
    endif
    l = $code_utils:parse_propref(dobjstr);
    if (l)
      dobj = player:match(l[1], player.location);
      if ($command_utils:object_match_failed(dobj, l[1]))
        return;
      endif
      prop = l[2];
      to_i = "to" in args;
      at_i = "at" in args;
      i = to_i && at_i ? min(to_i, at_i) | to_i || at_i;
      iobjstr = argstr[$string_utils:word_start(argstr)[i][2] + 1..$];
      iobjstr = $string_utils:trim(iobjstr);
      if (!iobjstr)
        try
          val = dobj.(prop) = "";
        except e (ANY)
          player:tell("Unable to set ", dobj, ".", prop, ": ", e[2]);
          return;
        endtry
        iobjstr = "\"\"";
        "elseif (iobjstr[1] == \"\\\"\")";
        "val = dobj.(prop) = iobjstr;";
        "iobjstr = \"\\\"\" + iobjstr + \"\\\"\";";
      else
        val = $string_utils:to_value(iobjstr);
        if (!val[1])
          player:tell("Could not parse: ", iobjstr);
          return;
        elseif (!$object_utils:has_property(dobj, prop))
          player:tell("That object does not define that property.");
          return;
        endif
        try
          val = dobj.(prop) = val[2];
        except e (ANY)
          player:tell("Unable to set ", dobj, ".", prop, ": ", e[2]);
          return;
        endtry
      endif
      player:tell("Property ", dobj, ".", prop, " set to ", $string_utils:print(val), ".");
    else
      player:tell("Property ", dobjstr, " not found.");
    endif
  endverb

  verb "build_option" (this none this) owner: #2 flags: "rxd"
    ":build_option(name)";
    "Returns the value of the specified builder option";
    if (caller == this || $perm_utils:controls(caller_perms(), this))
      return $options["build"]:get(this.build_options, args[1]);
    else
      return E_PERM;
    endif
  endverb

  verb "set_build_option" (this none this) owner: #2 flags: "rxd"
    ":set_build_option(oname,value)";
    "Changes the value of the named option.";
    "Returns a string error if something goes wrong.";
    if (!(caller == this || $perm_utils:controls(caller_perms(), this)))
      return tostr(E_PERM);
    endif
    "...this is kludgy, but it saves me from writing the same verb n times.";
    "...there's got to be a better way to do this...";
    verb[1..4] = "";
    foo_options = verb + "s";
    prop = verb[1..index(verb, "_") - 1];
    "...";
    if (typeof(s = $options[prop]:set(this.(foo_options), @args)) == STR)
      return s;
    elseif (s == this.(foo_options))
      return 0;
    else
      this.(foo_options) = s;
      return 1;
    endif
  endverb

  verb "@build-o*ptions @buildo*ptions @builder-o*ptions @buildero*ptions" (any any any) owner: #2 flags: "rd"
    "@<what>-option <option> [is] <value>   sets <option> to <value>";
    "@<what>-option <option>=<value>        sets <option> to <value>";
    "@<what>-option +<option>     sets <option>   (usually equiv. to <option>=1";
    "@<what>-option -<option>     resets <option> (equiv. to <option>=0)";
    "@<what>-option !<option>     resets <option> (equiv. to <option>=0)";
    "@<what>-option <option>      displays value of <option>";
    set_task_perms(player);
    what = "build";
    options = what + "_options";
    option_pkg = $options[what];
    set_option = "set_" + what + "_option";
    if (!args)
      player:notify_lines({"Current " + what + " options:", "", @option_pkg:show(this.(options), option_pkg.names)});
      return;
    elseif (typeof(presult = option_pkg:parse(args)) == STR)
      player:notify(presult);
      return;
    else
      if (length(presult) > 1)
        if (typeof(sresult = this:(set_option)(@presult)) == STR)
          player:notify(sresult);
          return;
        elseif (!sresult)
          player:notify("No change.");
          return;
        endif
      endif
      player:notify_lines(option_pkg:show(this.(options), presult[1]));
    endif
  endverb

  verb "@listedit @pedit" (any none none) owner: #36 flags: "rd"
    "@listedit|@pedit object.prop -- invokes the list editor.";
    "   if you are editing a list of strings, you're better off using @notedit.";
    $list_editor:invoke(dobjstr, verb);
  endverb

  verb "@classes" (any any any) owner: #2 flags: "rd"
    "$class_registry is in the following format:";
    "        { {name, description, members, parent}, ... }";
    "where `name' is the name of a particular class of objects, `description' is a one-sentence description of the membership of the class, `members' is a list of object numbers, the members of the class, and parent is a parent object in the player's hierarchy that may find the particular class interesting.";
    "";
    if (args)
      members = members_noroot = {};
      for name in (args)
        class = $list_utils:assoc_prefix(name, $class_registry);
        if (class)
          for o in (class[3])
            if (!isa(o, $root_class))
              members_noroot = setadd(members_noroot, o);
            else
              members = setadd(members, o);
            endif
          endfor
        else
          player:tell("There is no defined class of objects named `", name, "'; type `@classes' to see a complete list of defined classes.");
          return;
        endif
      endfor
      if (length(members) >= 30 || length(members_noroot) >= 30 && !$command_utils:yes_or_no("This command can be very spammy.  Are you certain you need this information?"))
        return player:tell("OK, aborting.  The lag thanks you.");
      endif
      printed = printed_noroot = {};
      for o in (members_noroot)
        what = o;
        while (valid(what))
          printed_noroot = setadd(printed_noroot, what);
          what = parent(what);
        endwhile
      endfor
      for o in (members)
        what = o;
        while (valid(what))
          printed = setadd(printed, what);
          what = parent(what);
        endwhile
      endfor
      player:tell("Members of the class", length(args) > 1 ? "es" | "", " named ", $string_utils:english_list(args), ":");
      player:tell();
      set_task_perms(player);
      if (printed)
        this:classes_2($root_class, "", members, printed);
      endif
      if (printed_noroot)
        this:classes_2($nothing, "", members_noroot, printed_noroot);
      endif
      player:tell();
    else
      "List all class names and descriptions";
      player:tell("The following classes of objects have been defined:");
      for class in ($class_registry)
        if (isa(player, class[4]))
          name = class[1];
          description = class[2];
          player:tell();
          player:tell("-- ", name, ": ", description);
        endif
      endfor
      player:tell();
      player:tell("Type `@classes <name>' to see the members of the class with the given <name>.");
    endif
  endverb

endobject
