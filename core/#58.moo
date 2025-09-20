object #58
  name: "Generic Programmer"
  parent: #4
  location: #-1
  owner: #2
  readable: true
  override "aliases" = {"Generic Programmer"};

  override "description" = "You see a player who is too experienced to have any excuse for not having a description.";

  override "features" = {#90, #75, #73, #9};

  override "home" = #62;

  property "eval_subs" (owner: #36, flags: "r") = {};

  property "eval_ticks" (owner: #36, flags: "r") = 3;

  property "eval_env" (owner: #36, flags: "r") = "here=player.location;me=player";

  property "prog_options" (owner: #2, flags: "rc") = {};

  verb "eval*-d" (any any any) owner: #2 flags: "rd"
    "A MOO-code evaluator.  Type `;CODE' or `eval CODE'.";
    "Calls player:eval_cmd_string to first transform CODE in any way appropriate (e.g., prefixing .eval_env) and then do the actual evaluation.  See documentation for this:eval_cmd_string";
    "If you set your .eval_time property to 1, you find out how many ticks and seconds you used.";
    "If eval-d is used, the evaluation is performed as if the debug flag were unset.";
    if (player != this)
      player:tell("I don't understand that.");
      return;
    elseif (!player.programmer)
      player:tell("You need to be a programmer to eval code.");
      return;
    endif
    set_task_perms(player);
    result = player:eval_cmd_string(argstr, verb != "eval-d");
    if (result[1])
      player:notify(this:eval_value_to_string(result[2]));
      if (player:prog_option("eval_time") && !`output_delimiters(player)[2] ! ANY')
        player:notify(tostr("[used ", result[3], " tick", result[3] != 1 ? "s, " | ", ", result[4], " second", result[4] != 1 ? "s" | "", ".]"));
      endif
    else
      player:notify_lines(result[2]);
      nerrors = length(result[2]);
      player:notify(tostr(nerrors, " error", nerrors == 1 ? "." | "s."));
    endif
  endverb

  verb "@kill @killq*uiet" (any none none) owner: #2 flags: "rd"
    "Kills one or more tasks.";
    "Arguments:";
    "   object:verb -- kills all tasks which were started from that object and verb.";
    "   all -- kills all tasks owned by invoker";
    "   all player-name -- wizard variant:  kills all tasks owned by player.";
    "   all everyone -- wizard variant:  really kills all tasks.";
    "   Integer taskid -- kills the specifically named task.";
    "   soon [integer] -- kills all tasks scheduled to run in the next [integer] seconds, which defaults to 60.";
    "   %integer -- kills all tasks which end in the digits contained in integer.";
    "   The @killquiet alias kills tasks without the pretty printout if more than one task is being killed.";
    set_task_perms(player);
    quiet = index(verb, "q");
    if (length(args) == 0)
      player:notify_lines({tostr("Usage:  ", verb, " [object]:[verb]"), tostr("        ", verb, " task_id"), tostr("        ", verb, " soon [number-of-seconds]", player.wizard ? " [everyone|<player name>]" | ""), tostr("        ", verb, " all", player.wizard ? " [everyone|<player name>]" | "")});
      return;
    elseif (taskid = toint(args[1]))
    elseif (all = args[1] == "all")
      everyone = 0;
      realplayer = player;
      if (player.wizard && length(args) > 1)
        realplayer = $string_utils:match_player(args[2]);
        everyone = args[2] == "everyone";
        if (!valid(realplayer) && !everyone)
          $command_utils:player_match_result(realplayer, args[2]);
          return;
        elseif (!everyone)
          set_task_perms(realplayer);
        endif
      endif
    elseif (soon = args[1] == "soon")
      realplayer = player;
      if (length(args) > 1)
        soon = toint(args[2]);
        if (soon <= 0 && !player.wizard)
          player:notify(tostr("Usage:  ", verb, " soon [positive-number-of-seconds]"));
          return;
        elseif (player.wizard)
          result = this:kill_aux_wizard_parse(@args[2..$]);
          soon = result[1];
          if (result[1] < 0)
            "already gave them an error message";
            return;
          elseif (result[2] == 1)
            everyone = 1;
          else
            everyone = 0;
            set_task_perms(result[2]);
            realplayer = result[2];
          endif
        endif
      else
        soon = 60;
        everyone = 0;
      endif
    elseif (percent = args[1][1] == "%")
      l = length(args[1]);
      digits = toint(args[1][2..l]);
      percent = toint("1" + "0000000000"[1..l - 1]);
    elseif (colon = index(argstr, ":"))
      whatstr = argstr[1..colon - 1];
      vrb = argstr[colon + 1..$];
      if (whatstr)
        what = player:match(whatstr);
      endif
    else
      player:notify_lines({tostr("Usage:  ", verb, " [object]:[verb]"), tostr("        ", verb, " task_id"), tostr("        ", verb, " soon [number-of-seconds]", player.wizard ? " [everyone|<player name>]" | ""), tostr("        ", verb, " all", player.wizard ? " [\"everyone\"|<player name>]" | "")});
      return;
    endif
    "OK, parsed the line, and punted them if it was bogus.  This verb could have been a bit shorter at the expense of readability.  I think it's getting towards unreadable as is.  At this point we've set_task_perms'd, and set up an enormous number of local variables.  Evaluate them in the order we set them, and we should never get var not found.";
    queued_tasks = queued_tasks();
    killed = 0;
    if (taskid)
      try
        kill_task(taskid);
        player:notify(tostr("Killed task ", taskid, "."));
        killed = 1;
      except error (ANY)
        player:notify(tostr("Can't kill task ", taskid, ": ", error[2]));
      endtry
    elseif (all)
      for task in (queued_tasks)
        if (everyone || realplayer == task[5])
          `kill_task(task[1]) ! ANY';
          killed = killed + 1;
          if (!quiet)
            this:_kill_task_message(task);
          endif
        endif
      endfor
    elseif (soon)
      now = time();
      for task in (queued_tasks)
        if (task[2] - now < soon && (!player.wizard || (everyone || realplayer == task[5])))
          `kill_task(task[1]) ! ANY';
          killed = killed + 1;
          if (!quiet)
            this:_kill_task_message(task);
          endif
        endif
      endfor
    elseif (percent)
      for task in (queued_tasks)
        if (digits == task[1] % percent)
          `kill_task(task[1]) ! ANY';
          killed = killed + 1;
          if (!quiet)
            this:_kill_task_message(task);
          endif
        endif
      endfor
    elseif (colon || vrb || whatstr)
      for task in (queued_tasks)
        if (whatstr == "" || (valid(task[6]) && index(task[6].name, whatstr) == 1) || (valid(task[9]) && index(task[9].name, whatstr) == 1) || task[9] == what || task[6] == what && (vrb == "" || index(" " + strsub(task[7], "*", ""), " " + vrb) == 1))
          `kill_task(task[1]) ! ANY';
          killed = killed + 1;
          if (!quiet)
            this:_kill_task_message(task);
          endif
        endif
      endfor
    else
      player:notify("Something is funny; I didn't understand your @kill command.  You shouldn't have gotten here.  Please send yduJ mail saying you got this message from @kill, and what you had typed to @kill.");
    endif
    if (!killed)
      player:notify("No tasks killed.");
    elseif (quiet)
      player:notify(tostr("Killed ", killed, " tasks."));
    endif
  endverb

  verb "_kill_task_message" (this none this) owner: #2 flags: "rxd"
    set_task_perms(caller_perms());
    task = args[1];
    player:notify(tostr("Killed: ", $string_utils:right(tostr("task ", task[1]), 17), ", verb ", task[6], ":", task[7], ", line ", task[8], task[9] != task[6] ? ", this==" + tostr(task[9]) | ""));
  endverb

  verb "@setenv" (any any any) owner: #2 flags: "rd"
    "Usage: @setenv <environment string>";
    "Set your .eval_env property.";
    set_task_perms(player);
    if (!argstr)
      player:notify(tostr("Usage:  ", verb, " <environment string>"));
      return;
    endif
    player:notify(tostr("Current eval environment is: ", player.eval_env));
    result = player:set_eval_env(argstr);
    if (typeof(result) == ERR)
      player:notify(tostr(result));
      return;
    endif
    player:notify(tostr(".eval_env set to \"", player.eval_env, "\" (", player.eval_ticks, " ticks)."));
  endverb

  verb "@d*isplay" (any none none) owner: #2 flags: "rd"
    "@display <object>[.[property]]*[,[inherited_property]]*[:[verb]]*[;[inherited_verb]]*";
    if (player != this)
      player:notify(tostr("Sorry, you can't use ", this:title(), "'s `", verb, "' command."));
      return E_PERM;
    endif
    "null names for properties and verbs are interpreted as meaning all of them.";
    opivu = {{}, {}, {}, {}, {}};
    string = "";
    punc = 1;
    literal = 0;
    set_task_perms(player);
    for jj in [1..length(argstr)]
      j = argstr[jj];
      if (literal)
        string = string + j;
        literal = 0;
      elseif (j == "\\")
        literal = 1;
      elseif (y = index(".,:;", j))
        opivu[punc] = {@opivu[punc], string};
        punc = 1 + y;
        string = "";
      else
        string = string + j;
      endif
    endfor
    opivu[punc] = {@opivu[punc], string};
    objname = opivu[1][1];
    it = this:match(objname);
    if ($command_utils:object_match_failed(it, objname))
      return;
    endif
    readable = it.owner == this || (it.r || this.wizard);
    cant = {};
    if ("" in opivu[2])
      if (readable)
        prop = properties(it);
      else
        prop = {};
        cant = setadd(cant, it);
      endif
      if (!this:display_option("thisonly"))
        what = it;
        while (!prop && valid(what = parent(what)))
          if (what.owner == this || (what.r || this.wizard))
            prop = properties(what);
          else
            cant = setadd(cant, what);
          endif
        endwhile
      endif
    else
      prop = opivu[2];
    endif
    if ("" in opivu[3])
      inh = {};
      for what in ({it, @$object_utils:ancestors(it)})
        if (what.owner == this || what.r || this.wizard)
          inh = {@inh, @properties(what)};
        else
          cant = setadd(cant, what);
        endif
      endfor
    else
      inh = opivu[3];
    endif
    for q in (inh)
      if (q in `properties(it) ! ANY => {}')
        prop = setadd(prop, q);
        inh = setremove(inh, q);
      endif
    endfor
    vrb = {};
    if ("" in opivu[4])
      if (readable)
        vrbs = verbs(it);
      else
        vrbs = $object_utils:accessible_verbs(it);
        cant = setadd(cant, it);
      endif
      what = it;
      if (!this:display_option("thisonly"))
        while (!vrbs && valid(what = parent(what)))
          if (what.owner == this || (what.r || this.wizard))
            vrbs = verbs(what);
          else
            cant = setadd(cant, what);
          endif
        endwhile
      endif
      for n in [1..length(vrbs)]
        vrb = setadd(vrb, {what, n});
      endfor
    else
      for w in (opivu[4])
        if (y = $object_utils:has_verb(it, w))
          vrb = setadd(vrb, {y[1], w});
        else
          this:notify(tostr("No such verb, \"", w, "\""));
        endif
      endfor
    endif
    if ("" in opivu[5])
      for z in ({it, @$object_utils:ancestors(it)})
        if (this == z.owner || z.r || this.wizard)
          for n in [1..length(verbs(z))]
            vrb = setadd(vrb, {z, n});
          endfor
        else
          cant = setadd(cant, z);
        endif
      endfor
    else
      for w in (opivu[5])
        if (typeof(y = $object_utils:has_verb(it, w)) == LIST)
          vrb = setadd(vrb, {y[1], w});
        else
          this:notify(tostr("No such verb, \"", w, "\""));
        endif
      endfor
    endif
    if ({""} in opivu || opivu[2..5] == {{}, {}, {}, {}})
      this:notify(tostr(it.name, " (", it, ") [ ", it.r ? "readable " | "", it.w ? "writeable " | "", it.f ? "fertile " | "", is_player(it) ? "(player) " | "", it.programmer ? "programmer " | "", it.wizard ? "wizard " | "", "]"));
      if (it.owner != (is_player(it) ? it | this))
        this:notify(tostr("  Owned by ", valid(p = it.owner) ? p.name | "** extinct **", " (", p, ")."));
      endif
      this:notify(tostr("  Child of ", valid(p = parent(it)) ? p.name | "** none **", " (", p, ")."));
      if (it.location != $nothing)
        this:notify(tostr("  Location ", valid(p = it.location) ? p.name | "** unplace (tell a wizard, fast!) **", " (", p, ")."));
      endif
    endif
    blankargs = this:display_option("blank_tnt") ? {"this", "none", "this"} | #-1;
    for b in (vrb)
      where = b[1];
      q = b[2];
      short = typeof(q) == INT ? q | strsub(y = index(q, " ") ? q[1..y - 1] | q, "*", "");
      inf = `verb_info(where, short) ! ANY';
      if (typeof(inf) == LIST || inf == E_PERM)
        name = typeof(inf) == LIST ? index(inf[3], " ") ? "\"" + inf[3] + "\"" | inf[3] | q;
        line = $string_utils:left(tostr($string_utils:right(tostr(where), 6), ":", name, " "), 32);
        if (inf == E_PERM)
          line = line + "   ** unreadable **";
        else
          line = $string_utils:left(tostr(line, inf[1].name, " (", inf[1], ") "), 53) + ((i = inf[2] in {"x", "xd", "d", "rd"}) ? {" x", " xd", "  d", "r d"}[i] | inf[2]);
          vargs = `verb_args(where, short) ! ANY';
          if (vargs != blankargs)
            if (this:display_option("shortprep") && !(vargs[2] in {"any", "none"}))
              vargs[2] = $code_utils:short_prep(vargs[2]);
            endif
            line = $string_utils:left(line + " ", 60) + $string_utils:from_list(vargs, " ");
          endif
        endif
        this:notify(line);
      elseif (inf == E_VERBNF)
        this:notify(tostr(inf));
        this:notify(tostr("  ** no such verb, \"", short, "\" **"));
      else
        this:notify("This shouldn't ever happen. @display is buggy.");
      endif
    endfor
    all = {@prop, @inh};
    max = length(all) < 4 ? 999 | this:linelen() - 56;
    depth = length(all) < 4 ? -1 | 1;
    truncate_owner_names = length(all) > 1;
    for q in (all)
      inf = `property_info(it, q) ! ANY';
      if (inf == E_PROPNF)
        if (q in $code_utils.builtin_props)
          this:notify(tostr($string_utils:left("," + q, 25), "Built in property            ", $string_utils:abbreviated_value(it.(q), max, depth)));
        else
          this:notify(tostr("  ** property not found, \"", q, "\" **"));
        endif
      else
        pname = $string_utils:left(tostr(q in `properties(it) ! ANY => {}' ? "." | (`is_clear_property(it, q) ! ANY' ? " " | ","), q, " "), 25);
        if (inf == E_PERM)
          this:notify(pname + "   ** unreadable **");
        else
          oname = inf[1].name;
          truncate_owner_names && (length(oname) > 12 && (oname = oname[1..12]));
          `inf[2][1] != "r" ! E_RANGE => 1' && (inf[2][1..0] = " ");
          `inf[2][2] != "w" ! E_RANGE => 1' && (inf[2][2..1] = " ");
          this:notify($string_utils:left(tostr($string_utils:left(tostr(pname, oname, " (", inf[1], ") "), 47), inf[2], " "), 54) + $string_utils:abbreviated_value(it.(q), max, depth));
        endif
      endif
    endfor
    if (cant)
      failed = {};
      for k in (cant)
        failed = listappend(failed, tostr(k.name, " (", k, ")"));
      endfor
      this:notify($string_utils:centre(tostr(" no permission to read ", $string_utils:english_list(failed, ", ", " or ", " or "), ". "), 75, "-"));
    else
      this:notify($string_utils:centre(" finished ", 75, "-"));
    endif
  endverb

  verb "set_eval_env" (this none this) owner: #36 flags: "rxd"
    "set_eval_env(string);";
    "Run <string> through eval.  If it doesn't compile, return E_INVARG.  If it crashes, well, it crashes.  If it works okay, set .eval_env to it and set .eval_ticks to the amount of time it took.";
    if (is_player(this) && $perm_utils:controls(caller_perms(), this))
      program = args[1];
      value = $no_one:eval_d(";ticks = ticks_left();" + program + ";return ticks - ticks_left() - 2;");
      if (!value[1])
        return E_INVARG;
      elseif (typeof(value[2]) == ERR)
        return value[2];
      endif
      try
        ok = this.eval_env = program;
        this.eval_ticks = value[2];
        return 1;
      except error (ANY)
        return error[1];
      endtry
    endif
  endverb

  verb "eval_cmd_string" (this none this) owner: #2 flags: "rxd"
    ":eval_cmd_string(string[,debug])";
    "Evaluates the string the way this player would normally expect to see it evaluated if it were typed on the command line.  debug (defaults to 1) indicates how the debug flag should be set during the evaluation.";
    " => {@eval_result, ticks, seconds}";
    "where eval_result is the result of the actual eval() call.";
    "";
    "For the case where string is an expression, we need to prefix `return ' and append `;' to string before passing it to eval().  However this is not appropriate for statements, where it is assumed an explicit return will be provided somewhere or that the return value is irrelevant.  The code below assumes that string is an expression unless it either begins with a semicolon `;' or one of the MOO language statement keywords.";
    "Next, the substitutions described by this.eval_subs, which should be a list of pairs {string, sub}, are performed on string";
    "Finally, this.eval_env is prefixed to the beginning while this.eval_ticks is subtracted from the eventual tick count.  This allows string to refer to predefined variables like `here' and `me'.";
    set_task_perms(caller_perms());
    {program, ?debug = 1} = args;
    program = program + ";";
    debug = debug ? 38 | 0;
    if (!match(program, "^ *%(;%|%(if%|fork?%|return%|while%|try%)[^a-z0-9A-Z_]%)"))
      program = "return " + program;
    endif
    program = tostr(this.eval_env, ";", $code_utils:substitute(program, this.eval_subs));
    ticks = ticks_left() - 53 - this.eval_ticks + debug;
    seconds = seconds_left();
    value = debug ? eval(program) | $code_utils:eval_d(program);
    seconds = seconds - seconds_left();
    ticks = ticks - ticks_left();
    return {@value, ticks, seconds};
  endverb

  verb "#*" (any any any) owner: #2 flags: "rd"
    "Copied from Player Class hacked with eval that does substitutions and assorted stuff (#8855):# by Geust (#24442) Sun May  9 20:19:05 1993 PDT";
    "#<string>[.<property>|.parent] [exit|player|inventory] [for <code>] returns information about the object (we'll call it <thing>) named by string.  String is matched in the current room unless one of exit|player|inventory is given.";
    "If neither .<property>|.parent nor <code> is specified, just return <thing>.";
    "If .<property> is named, return <thing>.<property>.  .parent returns parent(<thing>).";
    "If <code> is given, it is evaluated, with the value returned by the first part being substituted for %# in <code>.";
    "For example, the command";
    "  #JoeFeedback.parent player for toint(%#)";
    "will return 26026 (unless Joe has chparented since writing this).";
    set_task_perms(player);
    if (!(whatstr = verb[2..dot = min(index(verb + ".", "."), index(verb + ":", ":")) - 1]))
      player:notify("Usage:  #string [exit|player|inventory]");
      return;
    elseif (!args)
      what = player:match(whatstr);
    elseif (index("exits", args[1]) == 1)
      what = player.location:match_exit(whatstr);
    elseif (index("inventory", args[1]) == 1)
      what = player:match(whatstr);
    elseif (index("players", args[1]) == 1)
      what = $string_utils:match_player(whatstr);
      if ($command_utils:player_match_failed(what, whatstr))
        return;
      endif
    else
      what = player:match(whatstr);
    endif
    if (!valid(what) && match(whatstr, "^[0-9]+$"))
      what = toobj(whatstr);
    endif
    if ($command_utils:object_match_failed(what, whatstr))
      return;
    endif
    while (index(verb, ".parent") == dot + 1)
      what = parent(what);
      dot = dot + 7;
    endwhile
    if (dot >= length(verb))
      val = what;
    elseif ((value = $code_utils:eval_d(tostr("return ", what, verb[dot + 1..$], ";")))[1])
      val = value[2];
    else
      player:notify_lines(value[2]);
      return;
    endif
    if (prepstr)
      program = strsub(iobjstr + ";", "%#", toliteral(val));
      end = 1;
      "while (\"A\" <= (l = argstr[end]) && l <= \"Z\")";
      while ("A" <= (l = program[end]) && l <= "Z")
        end = end + 1;
      endwhile
      if (program[1] == ";" || program[1..end - 1] in {"if", "for", "fork", "return", "while", "try"})
        program = $code_utils:substitute(program, this.eval_subs);
      else
        program = $code_utils:substitute("return " + program, this.eval_subs);
      endif
      if ((value = eval(program))[1])
        player:notify(this:eval_value_to_string(value[2]));
      else
        player:notify_lines(value[2]);
        nerrors = length(value[2]);
        player:notify(tostr(nerrors, " error", nerrors == 1 ? "." | "s."));
      endif
    else
      player:notify(this:eval_value_to_string(val));
    endif
  endverb

  verb "eval_value_to_string" (this none this) owner: #2 flags: "rxd"
    set_task_perms(caller_perms());
    if (typeof(val = args[1]) == OBJ)
      return tostr("=> ", val, "  ", valid(val) ? "(" + val.name + ")" | ((a = $list_utils:assoc(val, {{#-1, "<$nothing>"}, {#-2, "<$ambiguous_match>"}, {#-3, "<$failed_match>"}})) ? a[2] | "<invalid>"));
    elseif (typeof(val) == ERR)
      return tostr("=> ", toliteral(val), "  (", val, ")");
    else
      return tostr("=> ", toliteral(val));
    endif
  endverb

  verb "@progo*ptions @prog-o*ptions @programmero*ptions @programmer-o*ptions" (any any any) owner: #2 flags: "rd"
    "@<what>-option <option> [is] <value>   sets <option> to <value>";
    "@<what>-option <option>=<value>        sets <option> to <value>";
    "@<what>-option +<option>     sets <option>   (usually equiv. to <option>=1";
    "@<what>-option -<option>     resets <option> (equiv. to <option>=0)";
    "@<what>-option !<option>     resets <option> (equiv. to <option>=0)";
    "@<what>-option <option>      displays value of <option>";
    set_task_perms(player);
    what = "prog";
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

  verb "prog_option" (this none this) owner: #2 flags: "rxd"
    ":prog_option(name)";
    "Returns the value of the specified prog option";
    if (caller in {this, $mcp.simpleedit} || $perm_utils:controls(caller_perms(), this))
      return $options["prog"]:get(this.prog_options, args[1]);
    else
      return E_PERM;
    endif
  endverb

  verb "set_prog_option" (this none this) owner: #2 flags: "rxd"
    ":set_prog_option(oname,value)";
    "Changes the value of the named option.";
    "Returns a string error if something goes wrong.";
    if (!(caller == this || $perm_utils:controls(caller_perms(), this)))
      return tostr(E_PERM);
    endif
    "...this is kludgy, but it saves me from writing the same verb 3 times.";
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

  verb "set_eval_subs" (none none none) owner: #2 flags: "rxd"
    "Copied from Player Class hacked with eval that does substitutions and assorted stuff (#8855):set_eval_subs by Geust (#24442) Fri Aug  5 13:18:59 1994 PDT";
    if (!$perm_utils:controls(caller_perms(), this))
      return E_PERM;
    elseif (typeof(subs = args[1]) != LIST)
      return E_TYPE;
    else
      for pair in (subs)
        if (length(pair) != 2 || typeof(pair[1] != STR) || typeof(pair[2] != STR))
          return E_INVARG;
        endif
      endfor
    endif
    return `this.eval_subs = subs ! ANY';
  endverb

  verb "@remove-option @rmoption @rm-option" (any from any) owner: #2 flags: "rd"
    if (!player.programmer)
      return E_PERM;
    endif
    set_task_perms(player);
    package = player:match(iobjstr);
    dobjstr = strsub(dobjstr, " ", "_");
    if (package == $failed_match || isa(package, $generic_options) == 0)
      return player:tell("You need to specify an option package.");
    elseif (dobjstr in package.names == 0)
      return player:tell("'", dobjstr, " isn't an option on ", $string_utils:nn(package), ".");
    elseif (!player.wizard && package.owner != player)
      return player:tell("You don't own that option package.");
    else
      if ($cu:yes_or_no(tostr("Really delete option '", dobjstr, " from ", $string_utils:nn(package), "?")) == 1)
        package:remove_name(dobjstr);
        delete_property(package, tostr("show_", dobjstr));
        player:tell("Option removed.");
      else
        player:tell("Aborted.");
      endif
    endif
  endverb

endobject
