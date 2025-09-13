object #73
  name: "Programmer Feature"
  parent: #74
  location: #83
  owner: #36
  readable: true

  override "aliases" = {"Programmer", "Feature", "Programmer Feature"};

  verb "@list*#" (any any any) owner: #2 flags: "rdx"
    "@list <obj>:<verb> [<dobj> <prep> <iobj>] [with[out] paren|num] [all] [ranges]";
    set_task_perms(player);
    bynumber = verb == "@list#";
    pflag = player:prog_option("list_all_parens");
    nflag = !player:prog_option("list_no_numbers");
    permflag = player:prog_option("list_show_permissions");
    aflag = 0;
    argspec = {};
    range = {};
    spec = args ? $code_utils:parse_verbref(args[1]) | E_INVARG;
    args = spec ? listdelete(args, 1) | E_INVARG;
    while (args)
      if (args[1] && (index("without", args[1]) == 1 || args[1] == "wo"))
        "...w,wi,wit,with => 1; wo,witho,withou,without => 0...";
        fval = !index(args[1], "o");
        if (`index("parentheses", args[2]) ! ANY' == 1)
          pflag = fval;
          args[1..2] = {};
        elseif (`index("numbers", args[2]) ! ANY' == 1)
          nflag = fval;
          args[1..2] = {};
        else
          player:notify(tostr(args[1], " WHAT?"));
          args = E_INVARG;
        endif
      elseif (index("all", args[1]) == 1)
        if (bynumber)
          player:notify("Don't use `all' with @list#.");
          args = E_INVARG;
        else
          aflag = 1;
          args[1..1] = {};
        endif
      elseif (index("0123456789", args[1][1]) || index(args[1], "..") == 1)
        if (E_INVARG == (s = $seq_utils:from_string(args[1])))
          player:notify(tostr("Garbled range:  ", args[1]));
          args = E_INVARG;
        else
          range = $seq_utils:union(range, s);
          args = listdelete(args, 1);
        endif
      elseif (bynumber)
        player:notify("Don't give args with @list#.");
        args = E_INVARG;
      elseif (argspec)
        "... second argspec?  Not likely ...";
        player:notify(tostr(args[1], " unexpected."));
        args = E_INVARG;
      elseif (typeof(pas = $code_utils:parse_argspec(@args)) == LIST)
        argspec = pas[1];
        if (length(argspec) < 2)
          player:notify(tostr("Argument `", @argspec, "' malformed."));
          args = E_INVARG;
        else
          argspec[2] = $code_utils:full_prep(argspec[2]) || argspec[2];
          args = pas[2];
        endif
      else
        "... argspec is bogus ...";
        player:notify(tostr(pas));
        args = E_INVARG;
      endif
    endwhile
    if (args == E_INVARG)
      if (bynumber)
        player:notify(tostr("Usage:  ", verb, " <object>:<verbnumber> [with|without parentheses|numbers] [ranges]"));
      else
        player:notify(tostr("Usage:  ", verb, " <object>:<verb> [<dobj> <prep> <iobj>] [with|without parentheses|numbers] [all] [ranges]"));
      endif
      return;
    elseif ($command_utils:object_match_failed(object = player:match(spec[1]), spec[1]))
      return;
    endif
    shown_one = 0;
    for what in ({object, @$object_utils:ancestors(object)})
      if (bynumber)
        vname = $code_utils:toint(spec[2]);
        if (vname == E_TYPE)
          return player:notify("Verb number expected.");
        elseif (vname < 1 || `vname > length(verbs(what)) ! E_PERM => 0')
          return player:notify("Verb number out of range.");
        endif
        code = `verb_code(what, vname, pflag) ! ANY';
      elseif (argspec)
        vnum = $code_utils:find_verb_named(what, spec[2]);
        while (vnum && `verb_args(what, vnum) ! ANY' != argspec)
          vnum = $code_utils:find_verb_named(what, spec[2], vnum + 1);
        endwhile
        vname = vnum;
        code = !vnum ? E_VERBNF | `verb_code(what, vnum, pflag) ! ANY';
      else
        vname = spec[2];
        code = `verb_code(what, vname, pflag) ! ANY';
      endif
      if (code != E_VERBNF)
        if (shown_one)
          player:notify("");
        elseif (what != object)
          player:notify(tostr("Object ", object, " does not define that verb", argspec ? " with those args" | "", ", but its ancestor ", what, " does."));
        endif
        if (typeof(code) == ERR)
          player:notify(tostr(what, ":", vname, " -- ", code));
        else
          info = verb_info(what, vname);
          vargs = verb_args(what, vname);
          fullname = info[3];
          if (index(fullname, " "))
            fullname = toliteral(fullname);
          endif
          if (index(vargs[2], "/"))
            vargs[2] = tostr("(", vargs[2], ")");
          endif
          player:notify(tostr(what, ":", fullname, "   ", $string_utils:from_list(vargs, " "), permflag ? " " + info[2] | ""));
          if (code == {})
            player:notify("(That verb has not been programmed.)");
          else
            lineseq = {1, length(code) + 1};
            range && (lineseq = $seq_utils:intersection(range, lineseq));
            if (!lineseq)
              player:notify("(No lines in that range.)");
            endif
            for k in [1..length(lineseq) / 2]
              for i in [lineseq[2 * k - 1]..lineseq[2 * k] - 1]
                if (nflag)
                  player:notify(tostr(" "[1..i < 10], i, ":  ", code[i]));
                else
                  player:notify(code[i]);
                endif
              endfor
            endfor
          endif
        endif
        shown_one = 1;
      endif
      if (shown_one && !aflag)
        return;
      endif
    endfor
    if (!shown_one)
      player:notify(tostr("That object does not define that verb", argspec ? " with those args." | "."));
    endif
  endverb

  verb "@chmod*#" (any any any) owner: #2 flags: "rdx"
    set_task_perms(player);
    bynumber = verb == "@chmod#";
    if (length(args) != 2)
      player:notify(tostr("Usage:  ", verb, " <object-or-property-or-verb> <permissions>"));
      return;
    endif
    {what, perms} = args;
    if (spec = $code_utils:parse_verbref(what))
      if (!player.programmer)
        player:notify("You need to be a programmer to do this.");
        player:notify("If you want to become a programmer, talk to a wizard.");
        return;
      endif
      if (valid(object = player:match(spec[1])))
        vname = spec[2];
        if (bynumber)
          vname = $code_utils:toint(vname);
          if (vname == E_TYPE)
            return player:notify("Verb number expected.");
          elseif (vname < 1 || `vname > length(verbs(object)) ! E_PERM => 0')
            return player:notify("Verb number out of range.");
          endif
        endif
        try
          info = verb_info(object, vname);
          if (!valid(owner = info[1]))
            player:notify(tostr("That verb is owned by an invalid object (", owner, "); it needs to be @chowned."));
          elseif (!is_player(owner))
            player:notify(tostr("That verb is owned by a non-player object (", owner.name, ", ", owner, "); it needs to be @chowned."));
          else
            info[2] = perms = $perm_utils:apply(info[2], perms);
            try
              result = set_verb_info(object, vname, info);
              player:notify(tostr("Verb permissions set to \"", perms, "\"."));
            except (E_INVARG)
              player:notify(tostr("\"", perms, "\" is not a valid permissions string for a verb."));
            except e (ANY)
              player:notify(e[2]);
            endtry
          endif
        except (E_VERBNF)
          player:notify("That object does not define that verb.");
        except error (ANY)
          player:notify(error[2]);
        endtry
        return;
      endif
    elseif (bynumber)
      return player:notify("@chmod# can only be used for verbs.");
    elseif (index(what, ".") && (spec = $code_utils:parse_propref(what)))
      if (valid(object = player:match(spec[1])))
        pname = spec[2];
        try
          info = property_info(object, pname);
          info[2] = perms = $perm_utils:apply(info[2], perms);
          try
            for target in ($ou:descendants(object))
              set_property_info(target, pname, info);
            endfor
            result = set_property_info(object, pname, info);
            player:notify(tostr("Property permissions set to \"", perms, "\"."));
          except (E_INVARG)
            player:notify(tostr("\"", perms, "\" is not a valid permissions string for a property."));
          except error (ANY)
            player:notify(error[2]);
          endtry
        except (E_PROPNF)
          player:notify("That object does not have that property.");
        except error (ANY)
          player:notify(error[2]);
        endtry
        return;
      endif
    elseif (valid(object = player:match(what)))
      perms = $perm_utils:apply((object.r ? "r" | "") + (object.w ? "w" | "") + (object.f ? "f" | ""), perms);
      r = w = f = 0;
      for i in [1..length(perms)]
        if (perms[i] == "r")
          r = 1;
        elseif (perms[i] == "w")
          w = 1;
        elseif (perms[i] == "f")
          f = 1;
        else
          player:notify(tostr("\"", perms, "\" is not a valid permissions string for an object."));
          return;
        endif
      endfor
      try
        object.r = r;
        object.w = w;
        object.f = f;
        player:notify(tostr("Object permissions set to \"", perms, "\"."));
      except (E_PERM)
        player:notify("Permission denied.");
      endtry
      return;
    endif
    $command_utils:object_match_failed(object, what);
  endverb

  verb "@rmprop*erty" (any any any) owner: #2 flags: "rdx"
    set_task_perms(player);
    if (length(args) != 1 || !(spec = $code_utils:parse_propref(args[1])))
      player:notify(tostr("Usage:  ", verb, " <object>.<property>"));
      return;
    endif
    object = player:match(spec[1]);
    pname = spec[2];
    if ($command_utils:object_match_failed(object, spec[1]))
      return;
    endif
    try
      result = delete_property(object, pname);
      player:rp_info_increment("PROPS-REMOVED");
      $broadcast:staff_alerts(player:name(), " removed ", pname, " from ", $su:nn(object), ".");
      player:notify("Property removed.");
    except (E_PROPNF)
      player:notify("That object does not define that property.");
    except res (ANY)
      player:notify(res[2]);
    endtry
  endverb

  verb "@rmverb*#" (any none none) owner: #2 flags: "rdx"
    set_task_perms(player);
    if (!(args && (spec = $code_utils:parse_verbref(args[1]))))
      player:notify(tostr("Usage:  ", verb, " <object>:<verb>"));
    elseif ($command_utils:object_match_failed(object = player:match(spec[1]), spec[1]))
      "...bogus object...";
    elseif (typeof(argspec = $code_utils:parse_argspec(@listdelete(args, 1))) != LIST)
      player:notify(tostr(argspec));
    elseif (argspec[2])
      player:notify($string_utils:from_list(argspec[2], " ") + "??");
    elseif (length(argspec = argspec[1]) in {1, 2})
      player:notify({"Missing preposition", "Missing iobj specification"}[length(argspec)]);
    else
      verbname = spec[2];
      if (verb == "@rmverb#")
        loc = $code_utils:toint(verbname);
        if (loc == E_TYPE)
          return player:notify("Verb number expected.");
        elseif (loc < 1 || loc > `length(verbs(object)) ! E_PERM => 0')
          return player:notify("Verb number out of range.");
        endif
      else
        if (index(verbname, "*") > 1)
          verbname = strsub(verbname, "*", "");
        endif
        loc = $code_utils:find_last_verb_named(object, verbname);
        if (argspec)
          argspec[2] = $code_utils:full_prep(argspec[2]) || argspec[2];
          while (loc != -1 && `verb_args(object, loc) ! ANY' != argspec)
            loc = $code_utils:find_last_verb_named(object, verbname, loc - 1);
          endwhile
        endif
        if (loc < 0)
          player:notify(tostr("That object does not define that verb", argspec ? " with those args." | "."));
          return;
        endif
      endif
      info = `verb_info(object, loc) ! ANY';
      vargs = `verb_args(object, loc) ! ANY';
      vcode = `verb_code(object, loc, 1, 1) ! ANY';
      try
        delete_verb(object, loc);
        if (info)
          player:notify(tostr("Verb ", object, ":", info[3], " (", loc, ") {", $string_utils:from_list(vargs, " "), "} removed."));
          $broadcast:staff_alerts(player:name(), " has removed verb ", object, ":", info[3], ".");
          if ("on_event_" in info[3] == 1)
            event_type = info[3][10..$];
            $event.subscribers[event_type] = setremove(`$event.subscribers[event_type] ! ANY => {}', object);
            if (!`$event.subscribers[event_type] ! ANY => {}')
              `$event.subscribers = mapdelete($event.subscribers, event_type) ! ANY';
              player:notify("  Event type ", event_type, " unsubscribed from $events system.");
            endif
          endif
          if (player:prog_option("rmverb_mail_backup"))
            $mail_agent:send_message(player, player, tostr(object, ":", info[3], " (", loc, ") {", $string_utils:from_list(vargs, " "), "}"), vcode);
          endif
        else
          player:notify(tostr("Unreadable verb ", object, ":", loc, " removed."));
        endif
      except e (ANY)
        player:notify(e[2]);
      endtry
    endif
  endverb

  verb "@args*#" (any any any) owner: #2 flags: "rdx"
    set_task_perms(player);
    if (!player.programmer)
      player:notify("You need to be a programmer to do this.");
      player:notify("If you want to become a programmer, talk to a wizard.");
      return;
    endif
    if (!(args && (spec = $code_utils:parse_verbref(args[1]))))
      player:notify(tostr(args ? "\"" + args[1] + "\"?  " | "", "<object>:<verb>  expected."));
    elseif ($command_utils:object_match_failed(object = player:match(spec[1]), spec[1]))
      "...can't find object...";
    else
      if (verb == "@args#")
        name = $code_utils:toint(spec[2]);
        if (name == E_TYPE)
          return player:notify("Verb number expected.");
        elseif (name < 1 || `name > length(verbs(object)) ! E_PERM => 0')
          return player:notify("Verb number out of range.");
        endif
      else
        name = spec[2];
      endif
      try
        info = verb_args(object, name);
        if (typeof(pas = $code_utils:parse_argspec(@listdelete(args, 1))) != LIST)
          "...arg spec is bogus...";
          player:notify(tostr(pas));
        elseif (!(newargs = pas[1]))
          player:notify($string_utils:from_list(info, " "));
        elseif (pas[2])
          player:notify(tostr("\"", pas[2][1], "\" unexpected."));
        else
          info[2] = info[2][1..index(info[2] + "/", "/") - 1];
          info = {@newargs, @info[length(newargs) + 1..$]};
          try
            result = set_verb_args(object, name, info);
            player:notify("Verb arguments changed.");
          except (E_INVARG)
            player:notify(tostr("\"", info[2], "\" is not a valid preposition (?)"));
          except error (ANY)
            player:notify(error[2]);
          endtry
        endif
      except (E_VERBNF)
        player:notify("That object does not have a verb with that name.");
      except error (ANY)
        player:notify(error[2]);
      endtry
    endif
  endverb

  verb "@copy @copy-x @copy-move" (any at any) owner: #2 flags: "rdx"
    "Usage:  @copy source:verbname to target[:verbname]";
    "  the target verbname, if not given, defaults to that of the source.  If the target verb doesn't already exist, a new verb is installed with the same args, names, code, and permission flags as the source.  Otherwise, the existing target's verb code is overwritten and no other changes are made.";
    "This the poor man's version of multiple inheritance... the main problem is that someone may update the verb you're copying and you'd never know.";
    "  if @copy-x is used, makes an unusable copy (!x, this none this).  If @copy-move is used, deletes the source verb as well.";
    set_task_perms(player);
    if (!player.programmer)
      player:notify("You need to be a programmer to do this.");
      player:notify("If you want to become a programmer, talk to a wizard.");
      return;
    elseif (!(from = $code_utils:parse_verbref(dobjstr)) || !iobjstr)
      player:notify(tostr("Usage:  ", verb, " obj:verb to obj:verb"));
      player:notify(tostr("        ", verb, " obj:verb to obj"));
      player:notify(tostr("        ", verb, " obj:verb to :verb"));
      return;
    elseif ($command_utils:object_match_failed(fobj = player:match(from[1]), from[1]))
      return;
    elseif (iobjstr[1] == ":")
      to = {fobj, iobjstr[2..$]};
    elseif (!index(iobjstr, ":") || !(to = $code_utils:parse_verbref(iobjstr)))
      iobj = player:match(iobjstr);
      if ($command_utils:object_match_failed(iobj, iobjstr))
        return;
      endif
      to = {iobj, from[2]};
    elseif ($command_utils:object_match_failed(tobj = player:match(to[1]), to[1]))
      return;
    else
      to[1] = tobj;
    endif
    from[1] = fobj;
    if (verb == "@copy-move")
      if (!$perm_utils:controls(player, fobj))
        player:notify("Won't be able to delete old verb.  Quota exceeded, so unable to continue.  Aborted.");
        return;
      elseif ($perm_utils:controls(player, fobj))
        "only try to move if the player controls the verb. Otherwise, skip and treat as regular @copy";
        if (typeof(result = $code_utils:move_verb(@from, @to)) == ERR)
          player:notify(tostr("Unable to move verb from ", from[1], ":", from[2], " to ", to[1], ":", to[2], " --> ", result));
        else
          player:notify(tostr("Moved verb from ", from[1], ":", from[2], " to ", result[1], ":", result[2]));
        endif
        return;
      else
        player:notify("Won't be able to delete old verb.  Treating this as regular @copy.");
      endif
    endif
    to_firstname = strsub(to[2][1..index(to[2] + " ", " ") - 1], "*", "") || "*";
    if (!(hv = $object_utils:has_verb(to[1], to_firstname)) || hv[1] != to[1])
      if (!(info = `verb_info(@from) ! ANY') || !(vargs = `verb_args(@from) ! ANY'))
        player:notify(tostr("Retrieving ", from[1], ":", from[2], " --> ", info && vargs));
        return;
      endif
      if (!player.wizard)
        info[1] = player;
      endif
      if (verb == "@copy-x")
        "... make sure this is an unusable copy...";
        info[2] = strsub(info[2], "x", "");
        vargs = {"this", "none", "this"};
      endif
      if (from[2] != to[2])
        info[3] = to[2];
      endif
      if (ERR == typeof(e = `add_verb(to[1], info, vargs) ! ANY'))
        player:notify(tostr("Adding ", to[1], ":", to[2], " --> ", e));
        return;
      endif
    endif
    code = `verb_code(@from) ! ANY';
    owner = `verb_info(@from)[1] ! ANY';
    if (typeof(code) == ERR)
      player:notify(tostr("Couldn't retrieve code from ", from[1].name, " (", from[1], "):", from[2], " => ", code));
      return;
    endif
    if (owner != player)
      if (!player:prog_option("copy_expert"))
        player:notify("Use of @copy is discouraged.  Please do not use @copy if you can use inheritance or features instead.  Use @copy carefully, and only when absolutely necessary, as it is wasteful of database space.");
      endif
    endif
    e = `set_verb_code(to[1], to_firstname, code) ! ANY';
    if (ERR == typeof(e))
      player:notify(tostr("Copying ", from[1], ":", from[2], " to ", to[1], ":", to[2], " --> ", e));
    elseif (typeof(e) == LIST && e)
      player:notify(tostr("Copying ", from[1], ":", from[2], " to ", to[1], ":", to[2], " -->"));
      player:notify_lines(e);
    else
      player:notify(tostr(to[1], ":", to[2], " code set."));
    endif
  endverb

  verb "@dump" (any any any) owner: #2 flags: "rdx"
    "@dump something [with [id=...] [noprops] [noverbs] [create]]";
    "This spills out all properties and verbs on an object, calling suspend at appropriate intervals.";
    "   id=#nnn -- specifies an idnumber to use in place of the object's actual id (for porting to another MOO)";
    "   noprops -- don't show properties.";
    "   noverbs -- don't show verbs.";
    "   create  -- indicates that a @create command should be generated and all of the verbs be introduced with @verb rather than @args; the default assumption is that the object already exists and you're just doing this to have a look at it.";
    set_task_perms(player);
    dobj = player:match(dobjstr);
    if ($command_utils:object_match_failed(dobj, dobjstr))
      return;
    endif
    if (prepstr && prepstr != "with")
      player:notify(tostr("Usage:  ", verb, " something [with [id=...] [noprops] [noverbs] [create]]"));
      return;
    endif
    targname = tostr(dobj);
    options = {"props", "verbs"};
    create = 0;
    if (iobjstr)
      for o in ($string_utils:explode(iobjstr))
        if (index(o, "id=") == 1)
          targname = o[4..$];
        elseif (o in {"noprops", "noverbs"})
          options = setremove(options, o[3..$]);
        elseif (o in {"create"})
          create = 1;
        else
          player:notify(tostr("`", o, "' not understood as valid option."));
          player:notify(tostr("Usage:  ", verb, " something [with [id=...] [noprops] [noverbs] [create]]"));
          return;
        endif
      endfor
    endif
    if (create)
      player:notify($code_utils:dump_preamble(dobj));
    endif
    if ("props" in options)
      player:notify_lines_suspended($code_utils:dump_properties(dobj, create, targname));
    endif
    if (!("verbs" in options))
      player:notify("\"***finished***");
      return;
    endif
    player:notify("");
    player:notify_lines_suspended($code_utils:dump_verbs(dobj, create, targname));
    player:notify("\"***finished***");
  endverb

  verb "@clearp*roperty @clprop*erty" (any none none) owner: #2 flags: "rdx"
    "@clearproperty <obj>.<prop>";
    "Set the value of <obj>.<prop> to `clear', making it appear to be the same as the property on its parent.";
    set_task_perms(player);
    if (!(l = $code_utils:parse_propref(dobjstr)))
      player:notify(tostr("Usage:  ", verb, " <object>.<property>"));
    elseif ($command_utils:object_match_failed(dobj = player:match(l[1]), l[1]))
      "... bogus object...";
    endif
    try
      if (is_clear_property(dobj, prop = l[2]))
        player:notify(tostr("Property ", dobj, ".", prop, " is already clear!"));
        return;
      endif
      clear_property(dobj, prop);
      player:notify(tostr("Property ", dobj, ".", prop, " cleared; value is now ", toliteral(dobj.(prop)), "."));
    except (E_INVARG)
      player:notify(tostr("You can't clear ", dobj, ".", prop, "; none of the ancestors define that property."));
    except error (ANY)
      player:notify(error[2]);
    endtry
  endverb

  verb "@verb" (any any any) owner: #2 flags: "rdx"
    set_task_perms(player);
    if (!player.programmer)
      player:notify("You need to be a programmer to do this.");
      player:notify("If you want to become a programmer, talk to a wizard.");
      return;
    endif
    if (!(args && (spec = $code_utils:parse_verbref(args[1]))))
      player:notify(tostr("Usage:  ", verb, " <object>:<verb-name(s)> [<dobj> [<prep> [<iobj> [<permissions> [<owner>]]]]]"));
      return;
    elseif ($command_utils:object_match_failed(object = player:match(spec[1]), spec[1]))
      return;
    endif
    name = spec[2];
    "...Adding another verb of the same name is often a mistake...";
    namelist = $string_utils:explode(name);
    for n in (namelist)
      if (i = index(n, "*"))
        n = n[1..i - 1] + n[i + 1..$];
      endif
      if ((hv = $object_utils:has_verb(object, n)) && hv[1] == object)
        player:notify(tostr("Warning:  Verb `", n, "' already defined on that object."));
      endif
    endfor
    if (typeof(pas = $code_utils:parse_argspec(@listdelete(args, 1))) != LIST)
      player:notify(tostr(pas));
      return;
    endif
    verbargs = pas[1] || (player:prog_option("verb_args") || {});
    verbargs = {@verbargs, "none", "none", "none"}[1..3];
    rest = pas[2];
    if (verbargs == {"this", "none", "this"})
      perms = player:prog_option("verb_perms") || "rxd";
      if (!index(perms, "x"))
        perms = perms + "x";
      endif
    else
      perms = player:prog_option("verb_perms") || "rd";
    endif
    if (rest)
      perms = $perm_utils:apply(perms, rest[1]);
    endif
    if (length(rest) < 2)
      owner = player;
    elseif (length(rest) > 2)
      player:notify(tostr("\"", rest[3], "\" unexpected."));
      return;
    elseif ($command_utils:player_match_result(owner = $string_utils:match_player(rest[2]), rest[2])[1])
      return;
    elseif (owner == $nothing)
      player:notify("Verb can't be owned by no one!");
      return;
    endif
    try
      x = add_verb(object, {#12, perms, name}, verbargs);
      player:notify(tostr("Verb added (", x > 0 ? x | length($object_utils:accessible_verbs(object)), ")."));
    except (E_INVARG)
      player:notify(tostr(rest ? tostr("\"", perms, "\" is not a valid set of permissions.") | tostr("\"", verbargs[2], "\" is not a valid preposition (?)")));
    except e (ANY)
      player:notify(e[2]);
    endtry
  endverb
  
  verb "@verbs @verbs/*" (any any any) owner: #2 flags: "rdx"
    if (!argstr)
      return this:ooc_tell("Syntax is @verbs <obj>.");
    endif
    object = player:match(argstr);
    if ($command_utils:object_match_failed(object, argstr))
      return;
    endif
    if ((switch = $su:explode(verb, "/")[$]) && switch != verb)
      if (!$ou:has_callable_verb(this, tostr("verbs_", switch)))
        switches = {};
        for v in ($ou:all_verbs(this))
          if (!$su:starts_with(v, "verbs_"))
            continue;
          endif
          switches = {@switches, v[7..$]};
        endfor
        return this:ooc_tell("Unable to match switch '", switch, "', available switches are: ", $su:english_list(switches));
      endif
      return this:(tostr("verbs_", switch))(object);
    endif
    return this:verbs_view(object);
  endverb

  verb "verbs_all" (this none this) owner: #2 flags: "rxd"
    return this:verbs_view(args[1], $ou:all_verbs(args[1]));
  endverb

  verb "verbs_view" (this none this) owner: #2 flags: "rxd"
    ":verbs_view(OBJ object[, LIST properties]) => NONE";
    "  Prints out a list of all verbs for player";
    set_task_perms(player);
    {object, ?all_verbs = verbs(object)} = args;
    if (!all_verbs)
      return player:tell("No verbs are defined on ", $su:nn(object), ".");
    endif
    player:tell("Verbs on ", $su:nn(object), " =>");
    idx = 1;
    lines = {$ansi:bright("IDX  NAME                                               SYS LEVEL  ARGUMENTS")};
    for verb_name in (all_verbs)
      try
        if (!$perm_utils:can_read_verb(player, object, verb_name))
          raise(E_PERM);
        endif
        {owner, perms, @more} = verb_info(object, verb_name);
      except e (ANY)
        lines = {@lines, tostr(" ", $su:left(idx, 4), $ansi:red(verb_name))};
        idx = idx + 1;
        continue;
      endtry
      defined = $ou:defines_verb(object, verb_name) ? " " | $ansi:brgreen(">");
      sys_level = $ansi:($perm_utils:can_write_verb(player, object, verb_name) ? "brgreen" | "brred")(owner:name());
      idx_col = $ansi:(idx % 2 ? "GreyCharcoal" | "GreyWheat")($su:left(idx, 4));
      lines = {@lines, tostr(" ", idx_col, $su:left(tostr(defined, $ansi:yellow(verb_name)), -51), $su:left(sys_level, 11), $su:from_list(verb_args(object, verb_name), " "))};
      idx = idx + 1;
    endfor
    player:tell_lines(lines);
    player:tell($ansi:brgreen(">"), " => Defined on Parent");
  endverb

  verb "@prog*ram @program#" (any any any) owner: #2 flags: "rdx"
    "This version of @program deals with multiple verbs having the same name.";
    "... @program <object>:<verbname> <dobj> <prep> <iobj>  picks the right one.";
    "...";
    "...catch usage errors first...";
    "...";
    set_task_perms(player);
    punt = "...set punt to 0 only if everything works out...";
    if (!(args && (spec = $code_utils:parse_verbref(args[1]))))
      player:notify(tostr("Usage: ", verb, " <object>:<verb> [<dobj> <prep> <iobj>]"));
    elseif ($command_utils:object_match_failed(object = player:match(spec[1]), spec[1]))
      "...bogus object...";
    elseif (typeof(argspec = $code_utils:parse_argspec(@listdelete(args, 1))) != LIST)
      player:notify(tostr(argspec));
    elseif (verb == "@program#")
      verbname = $code_utils:toint(spec[2]);
      if (verbname == E_TYPE)
        player:notify("Verb number expected.");
      elseif (length(args) > 1)
        player:notify("Don't give args for @program#.");
      elseif (verbname < 1 || `verbname > length(verbs(object)) ! E_PERM')
        player:notify("Verb number out of range.");
      else
        argspec = 0;
        punt = 0;
      endif
    elseif (argspec[2])
      player:notify($string_utils:from_list(argspec[2], " ") + "??");
    elseif (length(argspec = argspec[1]) in {1, 2})
      player:notify({"Missing preposition", "Missing iobj specification"}[length(argspec)]);
    else
      punt = 0;
      verbname = spec[2];
      if (index(verbname, "*") > 1)
        verbname = strsub(verbname, "*", "");
      endif
    endif
    "...";
    "...if we have an argspec, we'll need to reset verbname...";
    "...";
    if (punt)
    elseif (argspec)
      if (!(argspec[2] in {"none", "any"}))
        argspec[2] = $code_utils:full_prep(argspec[2]);
      endif
      loc = $code_utils:find_verb_named(object, verbname);
      while (loc > 0 && `verb_args(object, loc) ! ANY' != argspec)
        loc = $code_utils:find_verb_named(object, verbname, loc + 1);
      endwhile
      if (!loc)
        punt = "...can't find it....";
        player:notify("That object has no verb matching that name + args.");
      else
        verbname = loc;
      endif
    else
      loc = 0;
    endif
    "...";
    "...get verb info...";
    "...";
    if (punt || !(punt = "...reset punt to TRUE..."))
    else
      try
        info = verb_info(object, verbname);
        punt = 0;
        aliases = info[3];
        if (!loc)
          loc = aliases in (verbs(object) || {});
        endif
      except (E_VERBNF)
        player:notify("That object does not have that verb definition.");
      except error (ANY)
        player:notify(error[2]);
      endtry
    endif
    "...";
    "...read the code...";
    "...";
    if (punt)
      player:notify(tostr("Now ignoring code for ", args ? args[1] | "nothing in particular", "."));
      $command_utils:read_lines();
      player:notify("Verb code ignored.");
    else
      player:notify(tostr("Now programming ", object.name, ":", aliases, "(", !loc ? "??" | loc, ")."));
      lines = $command_utils:read_lines_escape({}, {tostr("You are editing ", $string_utils:nn(object), ":", verbname, ".")});
      try
        old_code = verb_code(object, verbname);
        if (result = set_verb_code(object, verbname, lines[2]))
          player:notify_lines(result);
          player:notify(tostr(length(result), " error(s)."));
          player:notify("Verb not programmed.");
        else
          `$diff_utils:diff_display("oldcode", old_code, "newcode", lines[2]) ! ANY';
          player:notify("0 errors.");
          player:notify("Verb programmed.");
          $code_scanner:display_issues($code_scanner:scan_for_issues(object, verbname));
        endif
      except error (ANY)
        player:notify(error[2]);
        player:notify("Verb not programmed.");
      endtry
    endif
  endverb
endobject