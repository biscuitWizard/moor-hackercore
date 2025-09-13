object #56
  name: "Command Utilities"
  parent: #78
  location: #-1
  owner: #2
  readable: true
  override "aliases" = {"Command Utilities"};

  override "description" = {"This is the command utilities utility package.  See `help $command_utils' for more details."};

  override "help_msg" = {"$command_utils is the repository for verbs that are of general usefulness to authors of all sorts of commands.  For more details about any of these verbs, use `help $command_utils:<verb-name>'.", "", "Detecting and Handling Failures in Matching", "-------------------------------------------", ":object_match_failed(match_result, name)", "    Test whether or not a :match_object() call failed and print messages if so.", ":player_match_failed(match_result, name)", "    Test whether or not a :match_player() call failed and print messages if so.", ":player_match_result(match_results, names)", "    ...similar to :player_match_failed, but does a whole list at once.", "", "Reading Input from the Player", "-----------------------------", ":read()         -- Read one line of input from the player and return it.", ":yes_or_no([prompt])", "                -- Prompt for and read a `yes' or `no' answer.", ":read_lines()   -- Read zero or more lines of input from the player.", ":dump_lines(lines) ", "                -- Return list of lines quoted so that feeding them to ", "                   :read_lines() will reproduce the original lines.", ":read_lines_escape(escapes[,help])", "                -- Like read_lines, except you can provide more escapes", "                   to terminate the read.", "", "Feature Objects", "---------------", ":validate_feature -- compare command line against feature verb argument spec", "", "Utilities for Suspending", "------------------------", ":running_out_of_time()", "                -- Return true if we're low on ticks or seconds.", ":suspend_if_needed(time)", "                -- Suspend (and return true) if we're running out of time.", "", "Client Support for Lengthy Commands", "-----------------------------------", ":suspend(args)  -- Handle PREFIX and SUFFIX for clients in long commands."};

  property "ABORT" (owner: #12, flags: "r") = 50;

  property "YES" (owner: #12, flags: "r") = 1;

  property "NO" (owner: #12, flags: "r") = 0;

  property "ALL" (owner: #12, flags: "r") = 2;

  property "NONE" (owner: #12, flags: "r") = -1;

  property "NO_ABORT" (owner: #12, flags: "r") = 100;

  property "feature_task" (owner: #36, flags: "") = "hey, neat, no feature verbs have been run yet!";

  verb "object_match_failed" (this none this) owner: #2 flags: "rxd"
    "Usage: object_match_failed(object, string, allow invalid)";
    "Prints a message if string does not match object.  Generally used after object is derived from a :match_object(string).";
    "If allow invalid is true, invalid objects > #0 are not considered a failure.";
    {match_result, string, ?allow_invalid = 0} = args;
    tell = $perm_utils:controls(caller_perms(), player) ? "notify" | "tell";
    if (index(string, "#") == 1 && $code_utils:toobj(string) != E_TYPE)
      "...avoid the `I don't know which `#-2' you mean' message...";
      if (allow_invalid && match_result > #0 && !valid(match_result))
        return 0;
      elseif (!valid(match_result))
        player:(tell)(tostr(string, " does not exist."));
      endif
      return !valid(match_result);
    elseif (match_result == $nothing)
      player:(tell)("You must give the name of some object.");
    elseif (match_result == $failed_match)
      player:(tell)(tostr("I see no \"", string, "\" here."));
    elseif (match_result == $ambiguous_match)
      player:(tell)(tostr("I don't know which \"", string, "\" you mean."));
    elseif (!allow_invalid && !valid(match_result))
      player:(tell)(tostr(match_result, " does not exist."));
    else
      return 0;
    endif
    return 1;
  endverb

  verb "player_match_result player_match_failed" (this none this) owner: #2 flags: "rxd"
    ":player_match_failed(result,string)";
    "  is exactly like :object_match_failed(result,string)";
    "  except that its messages are more suitable for player searches.";
    ":player_match_result(results,strings)";
    "  handles a list of results, also presumably from $string_utils:match_player(strings), printing messages to player for *each* of the nonmatching strings.  It returns a list, an overall result (true if some string didn't match --- just like player_match_failed), followed by the list players that matched.";
    "";
    "An optional 3rd arg gives an identifying string to prefix to each of the nasty messages.";
    if (valid(player))
      tell = $perm_utils:controls(caller_perms(), player) ? "notify" | "tell";
      plyr = player;
    else
      tell = "notify";
      plyr = $login;
    endif
    "...";
    {match_results, strings, ?cmdid = ""} = args;
    pmf = verb == "player_match_failed";
    if (typeof(match_results) == OBJ)
      match_results = {match_results};
      strings = {strings};
    endif
    pset = {};
    bombed = 0;
    for i in [1..length(match_results)]
      if (valid(result = match_results[i]))
        pset = setadd(pset, match_results[i]);
      elseif (result == $nothing)
        "... player_match_result quietly skips over blank strings";
        if (pmf)
          plyr:(tell)("You must give the name of some player.");
          bombed = 1;
        endif
      elseif (result == $failed_match)
        plyr:(tell)(tostr(cmdid, "\"", strings[i], "\" is not the name of any player."));
        bombed = 1;
      elseif (result == $ambiguous_match)
        lst = $player_db:find_all(strings[i]);
        plyr:(tell)(tostr(cmdid, "\"", strings[i], "\" could refer to ", length(lst) > 20 ? tostr("any of ", length(lst), " players") | $string_utils:english_list($list_utils:map_arg(2, $string_utils, "pronoun_sub", "%n (%#)", lst), "no one", " or "), "."));
        bombed = 1;
      else
        plyr:(tell)(tostr(result, " does not exist."));
        bombed = 1;
      endif
    endfor
    if (!bombed && !pset)
      "If there were NO valid results, but not any actual 'error', fail anyway.";
      plyr:(tell)("You must give the name of some player.");
      bombed = 1;
    endif
    return pmf ? bombed | {bombed, @pset};
  endverb

  verb "read" (this none this) owner: #36 flags: "rxd"
    "common usage:";
    "$command_utils:read([ 'a line of input' ]) -- read a line of input from the player and return it";
    "Optional argument is a prompt portion to replace `a line of input' in the prompt.";
    "";
    "connection reading:";
    "$command_utils:read('a line of input', connection)";
    "";
    "kill callback:";
    "$command_utils:read('a line of input', $nothing, verbname, arguments);";
    "The verbname defined on the kill callback object will be called with the supplied arguments";
    "Returns E_PERM if the current task is not a command task that has never called suspend().";
    {?prompt = "a line of input", ?connection = #-1, ?kill_verb = "", ?kill_args = {}} = args;
    c = callers();
    p = c[$][5];
    p:notify(tostr("[Type ", prompt, " or `@abort' to abort the command.]"));
    try
      if (connection != #-1)
        ans = read(connection);
      else
        if (!(p in connected_players()))
          raise(E_INVARG);
        endif
        ans = read();
      endif
      if ($string_utils:trim(ans) == "@abort")
        p:notify(">> Command Aborted <<");
        this:kill_task_callback(kill_verb, kill_args);
        kill_task(task_id());
      endif
      return ans;
    except error (E_INVARG)
      this:kill_task_callback(kill_verb, kill_args);
      kill_task(task_id());
    except error (ANY)
      return error[1];
    endtry
  endverb

  verb "read_lines" (this none this) owner: #36 flags: "rxd"
    "$command_utils:read_lines([max]) -- read zero or more lines of input";
    "";
    "Returns a list of strings, the (up to MAX, if given) lines typed by the player.  Returns E_PERM if the current task is not a command task that has never called suspend().";
    "In order that one may enter arbitrary lines, including \"@abort\" or \".\", if the first character in an input line is `.' and there is some nonwhitespace afterwords, the `.' is dropped and the rest of the line is taken verbatim, so that, e.g., \".@abort\" enters as \"@abort\" and \"..\" enters as \".\".";
    {?max = 0, ?kill_task_verb = "", ?kill_task_args = {}} = args;
    c = callers();
    p = c[$][5];
    p:notify(tostr("[Type", max ? tostr(" up to ", max) | "", " lines of input; use `.' (single period) on a separate line to end or `@abort' to abort the command.]"));
    ans = {};
    while (1)
      try
        line = read();
        if (line[1..min(6, $)] == "@abort" && (tail = line[7..$]) == $string_utils:space(tail))
          p:notify(">> Command Aborted <<");
          this:kill_task_callback(kill_task_verb, kill_task_args);
          kill_task(task_id());
        elseif (!line || line[1] != ".")
          ans = {@ans, line};
        elseif ((tail = line[2..$]) == $string_utils:space(tail))
          return ans;
        else
          ans = {@ans, tail};
        endif
        if (max && length(ans) >= max)
          return ans;
        endif
      except error (E_INVARG)
        this:kill_task_callback(kill_task_verb, kill_task_args);
        kill_task(task_id());
      except error (ANY)
        return error[1];
      endtry
    endwhile
  endverb

  verb "yes_or_no" (this none this) owner: #36 flags: "rxd"
    ":yes-or-no([prompt]) -- prompts the player for a yes or no answer and returns a true value iff the player enters a line of input that is some prefix of \"yes\"";
    "";
    "Returns E_NONE if the player enters a blank line, E_INVARG, if the player enters something that isn't a prefix of \"yes\" or \"no\", and E_PERM if the current task is not a command task that has never called suspend().";
    {?msg = "", ?abort_mode = this.ABORT, ?kill_verb = "", ?kill_args = {}} = args;
    c = callers();
    p = c[$][5];
    p:notify(tostr(msg ? msg + " " | "", "[Enter `yes' or `no']"));
    try
      dude = player;
      ans = read(@caller == p || $perm_utils:controls(caller_perms(), p) ? {p} | {});
      if (ans = $string_utils:trim(ans))
        if (abort_mode != this.NO_ABORT && ans == "@abort")
          p:notify(">> Command Aborted <<");
          callstack = c;
          {kill_caller, callstack} = $lu:pop(callstack);
          kill_caller = kill_caller[1];
          while (kill_caller == this && callstack)
            {kill_caller, callstack} = $lu:pop(callstack);
            kill_caller = kill_caller[1];
          endwhile
          if (kill_verb && $ou:has_callable_verb(kill_caller, kill_verb))
            try
              kill_caller:(kill_verb)(@kill_args);
            except e (ANY)
              $error:log(e);
            endtry
          endif
          kill_task(task_id());
        endif
        return index("yes", ans) == 1 || (index("no", ans) != 1 && E_INVARG);
      else
        return E_NONE;
      endif
    except error (E_INVARG)
      kill_task(task_id());
    except error (ANY)
      return error[1];
    endtry
  endverb

  verb "read_lines_escape" (this none this) owner: #2 flags: "rxd"
    "$command_utils:read_lines_escape(escapes[,help]) -- read zero or more lines of input";
    "";
    "Similar to :read_lines() except that help is available and one may specify other escape sequences to terminate the read.";
    "  escapes should be either a string or list of strings; this specifies which inputs other from `.' or `@abort' should terminate the read (... don't use anything beginning with a `.').";
    "  help should be a string or list of strings to be printed in response to the player typing `?'; the first line of the help text should be a general comment about what the input text should be used for.  Successive lines should describe the effects of the alternative escapes.";
    "Returns {end,list-of-strings-input} where end is the particular line that terminated this input or 0 if input terminated normally with `.'.  Returns E_PERM if the current task is not a command task that has never called suspend().  ";
    "@abort and lines beginning with `.' are treated exactly as with :read_lines()";
    {escapes, ?help = "You are currently in a read loop."} = args;
    c = callers();
    p = c[$][5];
    escapes = {".", "@abort", @typeof(escapes) == LIST ? escapes | {escapes}};
    p:notify(tostr("[Type lines of input; `?' for help; end with `", $string_utils:english_list(escapes, "", "' or `", "', `", ""), "'.]"));
    ans = {};
    escapes[1..0] = {"?"};
    "... set up the help text...";
    if (typeof(help) != LIST)
      help = {help};
    endif
    help[2..1] = {"Type `.' on a line by itself to finish.", "Anything else with a leading period is entered with the period removed.", "Type `@abort' to abort the command completely."};
    while (1)
      try
        line = read();
        if ((trimline = $string_utils:trimr(line)) in escapes)
          if (trimline == ".")
            return {0, ans};
          elseif (trimline == "@abort")
            p:notify(">> Command Aborted <<");
            kill_task(task_id());
          elseif (trimline == "?")
            p:notify_lines(help);
          else
            return {trimline, ans};
          endif
        else
          if (line && line[1] == ".")
            line[1..1] = "";
          endif
          ans = {@ans, line};
        endif
      except error (ANY)
        return error[1];
      endtry
    endwhile
  endverb

  verb "dump_lines" (this none this) owner: #36 flags: "rxd"
    ":dump_lines(text) => text `.'-quoted for :read_lines()";
    "  text is assumed to be a list of strings";
    "Returns a corresponding list of strings which, when read via :read_lines, ";
    "produces the original list of strings (essentially, any strings beginning ";
    "with a period \".\" have the period doubled).";
    "The list returned includes a final \".\"";
    text = args[1];
    newtext = {};
    i = lasti = 0;
    for line in (text)
      if (match(line, "^%(%.%| *@abort *$%)"))
        newtext = {@newtext, @i > lasti ? text[lasti + 1..i] | {}, "." + line};
        lasti = i = i + 1;
      else
        i = i + 1;
      endif
    endfor
    return {@newtext, @i > lasti ? text[lasti + 1..i] | {}, "."};
  endverb

  verb "explain_syntax" (this none this) owner: #2 flags: "rxd"
    ":explain_syntax(here,verb,args)";
    verb = args[2];
    for x in ({player, args[1], @valid(dobj) ? {dobj} | {}, @valid(iobj) ? {iobj} | {}})
      what = x;
      while (hv = $object_utils:has_verb(what, verb))
        what = hv[1];
        i = 1;
        while (i = $code_utils:find_verb_named(what, verb, i))
          if (evs = $code_utils:explain_verb_syntax(x, verb, @verb_args(what, i)))
            player:tell("Try this instead:  ", evs);
            return 1;
          endif
          i = i + 1;
        endwhile
        what = parent(what);
      endwhile
    endfor
    return 0;
  endverb

  verb "do_huh" (this none this) owner: #36 flags: "rx"
    ":do_huh(verb,args)  what :huh should do by default.";
    {verb, args} = args;
    this.feature_task = {task_id(), verb, args, argstr, dobj, dobjstr, prepstr, iobj, iobjstr};
    set_task_perms(cp = caller_perms());
    notify = $perm_utils:controls(cp, player) ? "notify" | "tell";
    testbit = player:my_huh(verb, args);
    if (testbit)
      "... the player found something funky to do ...";
    elseif (caller:here_huh(verb, args))
      "... the room found something funky to do ...";
    elseif (player:last_huh(verb, args))
      "... player's second round found something to do ...";
    elseif (dobj == $ambiguous_match)
      if (iobj == $ambiguous_match)
        player:(notify)(tostr("I don't understand that (\"", dobjstr, "\" and \"", iobjstr, "\" refer to multiple items 'help multiples')."));
      else
        player:(notify)(tostr("I don't understand that (\"", dobjstr, "\" refers to multiple items 'help multiples')."));
      endif
    elseif (iobj == $ambiguous_match)
      player:(notify)(tostr("I don't understand that (\"", iobjstr, "\" refers to multiple items 'help multiples')."));
    elseif (player.location:my_huh(verb, args))
      "... room found something to do ...";
    elseif (valid(dobj = dobj || player:match(dobjstr)) && dobj:my_huh(verb, args))
      "... dobj found something to do ...";
    elseif (valid(iobj = iobj || player:match(iobjstr)) && iobj:my_huh(verb, args))
      "... iobj found something to do ...";
    else
      player:(notify)($ru.idun_msg);
      player:my_explain_syntax(caller, verb, args) || (caller:here_explain_syntax(caller, verb, args) || this:explain_syntax(caller, verb, args));
    endif
  endverb

  verb "task_info" (this none this) owner: #2 flags: "rxd"
    "task_info(task id)";
    "Return info (the same info supplied by queued_tasks()) about a given task id, or E_INVARG if there's no such task queued.";
    "WIZARDLY";
    set_task_perms(caller_perms());
    tasks = queued_tasks();
    task_id = args[1];
    for task in (tasks)
      if (task[1] == task_id)
        return task;
      endif
    endfor
    return E_INVARG;
  endverb

  verb "validate_feature" (this none this) owner: #36 flags: "rxd"
    ":validate_feature(verb, args)";
    "  (where `verb' and `args' are the arguments passed to :my_huh)";
    "  returns true or false based on whether this is the same command typed by the user (comparing it against $command_utils.feature_task, set by $command_utils:do_huh).";
    "  assumes that the :my_huh parsing has not suspended";
    return {task_id(), @args, argstr, dobj, dobjstr, prepstr, iobj, iobjstr} == this.feature_task;
  endverb

  verb "reading_input" (this none this) owner: #2 flags: "rxd"
    "While input is being read() from a player, return 1. Otherwise return 0.";
    {who} = args;
    return `who.reading_input ! ANY => 0';
  endverb

  verb "switched_command" (this none this) owner: #36 flags: "rxd"
    {verbstr, cmd_prefix, ?default_verb = ""} = args;
    cmd_object = callers()[1][1];
    if ((switch = $su:explode(verbstr, "/")[$]) && switch != verbstr)
      if (!$ou:has_callable_verb(this, tostr(cmd_prefix, "_", switch)))
        switches = {};
        for v in ($ou:all_verbs(cmd_object))
          if (!$su:starts_with(v, tostr(cmd_prefix, "_")))
            continue;
          endif
          switches = {@switches, v[length(cmd_prefix)+1..$]};
        endfor
        player:notify(tostr("Unable to match switch '", switch, "', available switches are: ", $su:english_list(switches)));
        return $true;
      endif
      cmd_object:(tostr(cmd_prefix, "_", switch))(@args);
      return $true;
    endif
    if (default_verb)
      cmd_object:(default_verb)(@args);
      return $true;
    endif
    return $false;
  endverb

endobject
