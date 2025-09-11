object #99
  name: "ANSI Utilities"
  parent: #78
  location: #-1
  owner: #36
  readable: true
  override "key" = 0;

  override "aliases" = {"ANSI Utilities", "au"};

  override "description" = "{A utility for controlling ANSI sequences from within the MOO.  See 'help ansi-intro' for more info.}";

  override "help_msg" = {"A utility package for controlling ANSI sequences from within the MOO.  See 'help ansi-intro' for more info."};

  property "escape" (owner: #36, flags: "") = "";

  property "code_red" (owner: #36, flags: "r") = "e[31m";

  property "code_green" (owner: #36, flags: "r") = "e[32m";

  property "code_yellow" (owner: #36, flags: "r") = "e[33m";

  property "code_blue" (owner: #36, flags: "r") = "e[34m";

  property "code_purple" (owner: #36, flags: "r") = "e[35m";

  property "code_cyan" (owner: #36, flags: "r") = "e[36m";

  property "code_normal" (owner: #36, flags: "r") = "e[0m";

  property "code_inverse" (owner: #36, flags: "r") = "e[7m";

  property "code_underline" (owner: #36, flags: "r") = "e[4m";

  property "code_bold" (owner: #36, flags: "r") = "e[1m";

  property "code_b:black" (owner: #36, flags: "r") = "e[40m";

  property "code_b:red" (owner: #36, flags: "r") = "e[41m";

  property "code_b:green" (owner: #36, flags: "r") = "e[42m";

  property "code_b:yellow" (owner: #36, flags: "r") = "e[43m";

  property "code_b:blue" (owner: #36, flags: "r") = "e[44m";

  property "code_b:magenta" (owner: #36, flags: "r") = "e[45m";

  property "code_b:purple" (owner: #36, flags: "r") = "e[45m";

  property "code_b:cyan" (owner: #36, flags: "r") = "e[46m";

  property "code_b:white" (owner: #36, flags: "r") = "e[47m";

  property "beep" (owner: #36, flags: "") = "";

  property "code_bright" (owner: #36, flags: "r") = "e[1m";

  property "group_colors" (owner: #36, flags: "r") = {"red", "green", "yellow", "blue", "purple", "cyan", "gray", "grey", "magenta", "white"};

  property "group_bold" (owner: #36, flags: "r") = {"bold", "unbold", "bright", "unbright"};

  property "code_unbold" (owner: #36, flags: "r") = "e[22m";

  property "group_misc" (owner: #36, flags: "r") = {"underline", "inverse", "strike", "italic"};

  property "code_blink" (owner: #36, flags: "r") = "e[5m";

  property "code_unblink" (owner: #36, flags: "r") = "e[25m";

  property "group_blinking" (owner: #36, flags: "r") = {"blink", "unblink"};

  property "code_magenta" (owner: #36, flags: "r") = "e[35m";

  property "code_unbright" (owner: #36, flags: "r") = "e[22m";

  property "groups" (owner: #36, flags: "r") = {"bold", "colors", "misc", "blinking", "extra"};

  property "code_white" (owner: #36, flags: "r") = "e[37m";

  property "noansi_task" (owner: #36, flags: "r") = 828515096;

  property "all" (owner: #36, flags: "r") = {"random", "normal", "null", "bold", "unbold", "bright", "unbright", "red", "green", "yellow", "blue", "purple", "cyan", "gray", "grey", "magenta", "white", "underline", "inverse", "strike", "italic", "blink", "unblink", "beep"};

  property "all_regexp" (owner: #36, flags: "r") = "%[%(b:%)?%(random%|normal%|null%|bold%|unbold%|bright%|unbright%|red%|green%|yellow%|blue%|purple%|cyan%|gray%|grey%|magenta%|white%|underline%|inverse%|strike%|italic%|blink%|unblink%|beep%|:%([01][0-9][0-9]%|2[0-4][0-9]%|25[0-5]%)%|:random%|%([0-9]+%):%([0-9]+%):%([0-9]+%)%)%]";

  property "group_colors_regexp" (owner: #36, flags: "r") = "%[%(red%|green%|yellow%|blue%|purple%|cyan%|gray%|grey%|magenta%|white%)%]";

  property "group_bold_regexp" (owner: #36, flags: "r") = "%[%(bold%|unbold%|bright%|unbright%)%]";

  property "group_misc_regexp" (owner: #36, flags: "r") = "%[%(underline%|inverse%|strike%|italic%)%]";

  property "group_blinking_regexp" (owner: #36, flags: "r") = "%[%(blink%|unblink%)%]";

  property "truecolor_regexp" (owner: #36, flags: "r") = "%([0-9]+%):%([0-9]+%):%([0-9]+%)";

  property "version" (owner: #36, flags: "r") = "2.7";

  property "code_gray" (owner: #36, flags: "r") = "e[1;30m";

  property "code_grey" (owner: #36, flags: "r") = "e[1;30m";

  property "test_screen" (owner: #36, flags: "r") = {"Colors        [red]red[normal]          Bold Colors   [bold][red]red[normal]", "              [green]green[normal]                      [bold][green]green[normal]", "              [blue]blue[normal]                       [bold][blue]blue[normal]", "              [yellow]yellow[normal]                     [bold][yellow]yellow[normal]", "              [cyan]cyan[normal]                       [bold][cyan]cyan[normal]", "              [purple]purple[normal]                     [bold][purple]purple[normal]", "              [gray]gray[normal]                       [bold][gray]gray[normal]", "              [white]white[normal]                      [bold][white]white[normal]", "", "Backgrounds   [b:red][gray]red[normal]          True Color    [128:0:0]maroon[normal]", "              [b:green][gray]green[normal]                      [153:255:180]light green[normal]", "              [b:blue][gray]blue[normal]                       [153:204:255]light blue[normal]", "              [b:yellow][gray]yellow[normal]                     [255:255:153]funky banana[normal]", "              [b:cyan][gray]cyan[normal]                       [0:102:102]dark cyan[normal]", "              [b:purple][gray]purple[normal]                     [255:204:242]light purple[normal]", "              [b:white][gray]white[normal]                      [170:121:65]brown[normal]", "", "256 Color     [:130]Orange[normal]", "              [:125]Pink[normal]", "              [:055]Violet[normal]", "	      [:102]Silver[normal]", "	      [:178]Bold Tan[normal]", "	      [:118]Bold Lime[normal]", "	      ", "Blinking   - [blink]This text should be blinking.  [unblink]This shouldn't.[normal]", "", "Bold       - [bold][cyan]This text should be bold.  [unbold][cyan]This shouldn't.[normal]", "", "Inverse    - [inverse]This should be inverse, and [red]t[green]h[yellow]i[blue]s[normal][inverse] should be inverse and in color.[normal]", "", "Underline  - [underline]This should be underlined.[normal]", "", "Strike     - [strike]This should have a line through it.[normal]", "", "Italic     - [italic]This should be italicized.[normal]", "", "Random     - [random]All [random]of [random]these [random]words [random]should [random]be [random]written [random]in [random]a [random]different [random]color.[normal]", "", "Random 256 - [:random]All [:random]of [:random]these [:random]words [:random]should [:random]be [:random]written [:random]in [:random]a [:random]different [:random]color.[normal]", "", "Beep       - [beep]You should hear a beep."};

  property "extra_codes" (owner: #36, flags: "r") = {"random", "normal", "null"};

  property "active" (owner: #36, flags: "r") = 1;

  property "random_colors" (owner: #36, flags: "r") = {"red", "green", "yellow", "blue", "purple", "cyan", "gray", "white"};

  property "ansi_log" (owner: #36, flags: "r") = {{"2.7", 1713938968, "[bold][yellow]l[normal][yellow]i[green]s[bold]d[red]u[normal][red]d[purple]e[normal]", {"Fix a traceback in $ansi_utils:add_code()", "Add [strike[null]] for strikethrough text.", "Add [italic[null]] for italicized text."}}, {"2.6", 1637293016, "[bold][yellow]l[normal][yellow]i[green]s[bold]d[red]u[normal][red]d[purple]e[normal]", {"Fixed an issue in $ansi_pc:notify where it would terminate_normal even if the task was in the noansi queue, resulting in lines with visible ANSI tags gaining a [normal[null]] tag at the end.", "Made @dump add the task to the noansi queue, allowing ANSI tags to be visible.", "Added a call to 'remove_noansi' to @list to cleanup the noansi queue property."}}, {"2.5", 1550344231, "[bold][yellow]l[normal][yellow]i[green]s[bold]d[red]u[normal][red]d[purple]e[normal]", {"Added support for xterm 256 colors. Tags are in the mildly awkward form: [:number] or [b::number] for backgrounds. e.g. [:200[null]]", "Also merged True Color into the notify_regexp and removed background codes. Backgrounds are now a modifier to each color class."}}, {"2.4", 1540863892, "[bold][yellow]l[normal][yellow]i[green]s[bold]d[red]u[normal][red]d[purple]e[normal]", {"Added support for true color (24-bit) terminals. Tags are in the form: [red:green:blue] or [b:red:green:blue] for backgrounds"}}, {"2.3", 1112950415, "[bold][yellow]l[normal][yellow]i[green]s[bold]d[red]u[normal][red]d[purple]e[normal]", {"Added Remco de Groot's 2.2 change to the ANSI log and added support for background colors. Tags for background colors take the form: [b:color]", "Also updated @ansi-setup to recognize and use FileIO if it's available."}}, {"2.2", 820519200, "Remco de Groot", {"Fixed a bug in the setup script where it referenced $player rather than player."}}, {"2.1", 820519200, "[cyan]D[gray]ark_[unbold][cyan]O[gray]wl[normal]", {"Updated all the documentation. Fixed up $ansi_pc:@ansi-setup. Fixed up $ansi_utils:notify to be a *much* faster. Changed the format of $ansi_utils.ansi_log and made $ansi_help:ansi_log do the formatting itself. Added $ansi_utils:tell, $ansi_pc:title, $ansi_utils:ansi_title, help ansi-cutoff, $ansi_pc:confunc, $ansi_utils:quote_ansi, $ansi_utils.status_message, $ansi_pc:@ansi-status, help @ansi-status, $ansi_utils:trusts, $ansi_utils.trusted, $ansi_utils:setadd, $ansi_utils:setremove, $ansi_utils:terminate_normal, $ansi_utils:self_diagnostic, and a benchmark test. Fixed a bug in $ansi_utils:replace_group where it was ignoring @ansi-o escape. Removed $ansi_utils:maybe_restart_noansi and replaced it with the less server intensive :cleanup_noansi. Moved the main part of $ansi_utils:cutoff into an internal verb, :cutoff_locs, and made a :cutoff_assign verb that also uses it. Removed $ansi_utils:replace_group and :reset_string and moved it to :notify. Got rid of $ansi_utils.su_<whatever> and moved them to regular verbs on $ansi_utils. Fixed up $ansi_pc:linesplit to use $ansi_utils:cutoff_locs. Renamed $ansi_utils:update_regexp to update_all and made it update all the caches. Moved @ansi-setup to $ansi_utils.", "NOTE: $ansi_utils:notify doesn't terminate all strings with a [[null]normal] code anymore, you will have to run the string through $ansi_utils:terminate_normal before calling it.", "The home of the ANSI system is now NestMOO, instead of EnigMOO."}}, {"2.0", 817840800, "[red]D[gray]ark_[unbold][red]O[gray]wl[normal]", "Updated all the documentation, removed $ansi_utils.test_title, added $ansi_pc:set_aliases, added $ansi_pc:@more to fix the one on $player, fixed up @ansi-setup a lot. I'm fairly confident that the ANSI system is now safe and secure."}, {"1.6", 817840800, "[blue]D[gray]ark_[unbold][blue]O[gray]wl[normal]", "Made $ansi_utils:notify, moved all the replacing stuff there, and hacked $ansi_pc:notify to call it instead of notify(). Moved the .noansi_queue, :add_noansi, and :remove_noansi from $ansi_pc to $ansi_utils. I also updated 'help ansi-bugs' now that I tracked down a couple of them."}, {"1.5", 815248800, "[green]Grant[normal]", "Fixed some security problems that allowed players to get a copy of the escape character and use it for malicious purposes. To do that, I got rid of $ansi_pc:replace_ansi and put the code straight in :notify, which is able to pass() now. Also made an @ansi-setup verb for convenient porting, along with some core verb code on $ansi_utils. Fixed randoms to substitute faster by adding .random_colors. Added $ansi_utils:show_who_listing and changed @who to use it."}, {"1.4", 815248800, "[green]Grant[normal]", "In Dark_Owl's absence, I fixed up a lot of code to make it take less ticks, due to the recent lag problems on EnigMOO, which we suspect were called by color codes (most noticeably s). Average :notify call with colors takes around 200 less ticks. Also updated much of the documentation, since I know DO doesn't like to do it ;) Copied the part of $player:set_name that checked whether people had color codes in their name or not to the ANSI PC."}, {"1.3", 812570400, "[green]D[gray]ark_[unbold][green]O[gray]wl[normal]", "Fixed up 'help ansi-programming' a little more. Fixed $string_utils:columnize, for some reason it worked fine here but got screwed up on ForestMOO. Added 'help ansi-porting'."}, {"1.2", 812570400, "[purple]D[gray]ark_[unbold][purple]O[gray]wl[normal]", "$ansi_utils:replace_group now uses substitute(). Added 'help ansi-programming' and 'help ansi-bugs'. Added @ansi-test and 'help @ansi-test'. Cleaned up a lot of other stuff. Added $ansi_utils.active."}, {"1.1", 812570400, "[green]D[gray]ark_[unbold][green]O[gray]wl[normal]", "Fixed up $ansi_pc:notify a little but unfortunately now it requires wizperms and overrides the ones on it's ancestors. Fixed the line wrapping. Added this log, along with $ansi_utils.version. Hacked $ansi_utils:replace_group and $ansi_utils:cutoff to use regexps."}, {"1.0", 809978400, "[cyan]D[gray]ark_[unbold][cyan]O[gray]wl[normal]", "$emu disappears and is replaced with $ansi_utils. $ansi_pc, $ansi_help, $ansi_options, are created and everything on $player is moved to $ansi_pc. @ansi is replaced by @ansi-o."}, {"0.9", 807300000, "[white]D[gray]ark_[unbold][white]O[gray]wl[normal]", "EnigMOO opens resulting in a lot of bug fixes and more core hacks."}, {"0.6", 799351200, "[cyan]D[gray]ark_[unbold][cyan]O[gray]wl[normal]", "Fixed up @ansi some more, fixed a lot more core stuff including $generic_editor, and made it possible to ignore ANSI codes."}, {"0.4", 788983200, "[yellow]D[gray]ark_[unbold][yellow]O[gray]wl[normal]", "Fixed up @ansi a little and hacked some core stuff, mostly on $string_utils to fix columnizing. $emu:replace is hacked to use strsub() as opposed to going through letter by letter, ick."}, {"0.2", 781034400, "[purple]D[gray]ark_[unbold][purple]O[gray]wl[normal]", "Added an .ansi_on property on $player and a primitive @ansi command to turn it on and off."}, {"0.1", 778442400, "[white]D[gray]ark_[unbold][white]O[gray]wl[normal]", "First version, added $emu and hacked $player:notify to call it."}};

  property "noansi_queue" (owner: #36, flags: "r") = {414639345};

  property "need_wizperms" (owner: #36, flags: "r") = {{"ansi_pc", "@more"}, {"ansi_pc", "notify"}, {"ansi_utils", "notify"}, {"ansi_utils", "@ansi-setup"}};

  property "code_beep" (owner: #36, flags: "r") = "b";

  property "status_message" (owner: #36, flags: "r") = "";

  property "trusted" (owner: #36, flags: "r") = {};

  property "diagnostic_tests" (owner: #36, flags: "r") = {"benchmark"};

  property "redirect_su_names" (owner: #36, flags: "r") = {"left", "right", "center centre", "columnize columnise", "space"};

  property "redirect_su_code" (owner: #36, flags: "r") = {"\"...redirects verbs to $ansi_utils...\";", "if (verb == \"redirect_ansi\")", "elseif (valid(au = $ansi_utils))", "  return au:(verb)(@args);", "else", "  return this:(verb + \"(noansi)\")(@args);", "endif"};

  property "default_codes" (owner: #36, flags: "r") = {"white", "unbold", "unblink", "null"};

  property "code_null" (owner: #36, flags: "r") = "";

  property "code_random" (owner: #36, flags: "r") = "";

  property "group_extra" (owner: #36, flags: "r") = {"beep"};

  property "group_extra_regexp" (owner: #36, flags: "r") = "%[%(beep%)%]";

  property "terminate_regexp" (owner: #36, flags: "rc") = "%[%(b:%)?%(random%|normal%|bold%|bright%|unbright%|red%|green%|yellow%|blue%|purple%|cyan%|gray%|grey%|magenta%|underline%|inverse%|strike%|italic%|blink%|:%([01][0-9][0-9]%|2[0-4][0-9]%|25[0-5]%)%|:random%|%([0-9]+%):%([0-9]+%):%([0-9]+%)%)%]";

  property "replace_code_pointers" (owner: #36, flags: "r") = {};

  property "notify_regexp" (owner: #36, flags: "rc") = "%[%(b:%)?%(random%|normal%|bold%|unbold%|bright%|unbright%|red%|green%|yellow%|blue%|purple%|cyan%|gray%|grey%|magenta%|white%|underline%|inverse%|strike%|italic%|blink%|unblink%|beep%|:%([01][0-9][0-9]%|2[0-4][0-9]%|25[0-5]%)%|:random%|%([0-9]+%):%([0-9]+%):%([0-9]+%)%)%]";

  property "reset_guest_props" (owner: #36, flags: "r") = {"ansi_options", "replace_codes"};

  property "ge_fill_string" (owner: #36, flags: "r") = {"if (valid(au = $ansi_utils) && au.active)", "  return au:(verb)(@args);", "else", "  return this:(verb + \"(noansi)\")(@args);", "endif"};

  property "plr_db_insert" (owner: #36, flags: "r") = {"typeof(args[1]) == NUM && typeof(args[2]) == STR && (args[2] = $ansi_utils:delete(args[2]));", "typeof(args[1]) == STR && (args[1] = $ansi_utils:delete(args[1]));", "return pass(@args);"};

  property "xterm_256_regexp" (owner: #36, flags: "rc") = ":%([01][0-9][0-9]%|2[0-4][0-9]%|25[0-5]%)%|:random";

  property "code_strike" (owner: #36, flags: "r") = "e[9m";

  property "code_italic" (owner: #36, flags: "r") = "e[3m";

  verb "length" (this none this) owner: #36 flags: "rxd"
    return length(index(a = args[1], "[") ? this:delete(a) | a);
  endverb

  verb "index rindex" (this none this) owner: #36 flags: "rxd"
    ":[r]index (STR string, STR character, NUM case_matters)";
    "like index() and rindex() but ignores ANSI codes";
    return verb == "index" ? index(this:delete(args[1]), @listdelete(args, 1)) | rindex(this:delete(args[1]), @listdelete(args, 1));
  endverb

  verb "contains_codes" (this none this) owner: #36 flags: "rx"
    ":contains_codes(STR string) => True if <string> contains any ANSI codes";
    return !!match(args[1], this.all_regexp);
  endverb

  verb "delete" (this none this) owner: #36 flags: "rxd"
    ":delete (STR string) => STR <string> with ANSI codes stripped out";
    line = args[1];
    if (this.active)
      while (index = match(line, this.notify_regexp))
        line[index[1]..index[2]] = "";
      endwhile
      line = strsub(line, "[null]", "");
    endif
    return line;
  endverb

  verb "get_code" (this none this) owner: #36 flags: "rx"
    if (caller != this)
      return E_PERM;
    endif
    {code, escape_char, ?truecolor_match = 0, ?xterm_256_match = 0} = args;
    if (truecolor_match)
      ret = tostr(escape_char || this.escape, "[", code[1] == "b" ? "48" | "38");
      ret = substitute(tostr(ret, ";2;%4;%5;%6m"), truecolor_match);
      return ret;
    elseif (xterm_256_match)
      ret = tostr(escape_char || this.escape, "[", code[1] == "b" ? "48" | "38");
      ret = tostr(ret, ";5;", code[code[1] == "b" ? 4 | 2..$], "m");
      return ret;
    else
      return strsub(strsub(this.("code_" + code), "e", escape_char || this.escape, 1), "b", this.beep);
    endif
  endverb

  verb "cutoff*" (this none this) owner: #36 flags: "rx"
    ":cutoff (STR string, NUM start, NUM end) => STR";
    "Acts like: string[start..end] but ignores ANSI codes.";
    args = {@args, 0}[1..4];
    if (typeof(info = this:cutoff_locs(@args, 0)) == LIST)
      return args[1][info[1]..info[2]];
    else
      return info;
    endif
  endverb

  verb "add_group" (this none this) owner: #36 flags: "rxd"
    ":add_group (STR group)";
    "Adds <group> to the groups and makes a property for it.";
    if (!this:trusts(caller_perms()))
      return E_PERM;
    elseif (!(args && args[1] && typeof(args[1]) == STR))
      return E_INVARG;
    elseif (args[1] in this.groups)
      return 0;
    else
      this.groups = setadd(this.groups, args[1]);
      arg1 = {this, "group_" + args[1], {}, {$code_utils:verb_perms(), "r"}};
      arg2 = {this, tostr("group_", args[1], "_regexp"), "", arg1[4]};
      if ($object_utils:has_callable_verb(#0, "add_property"))
        $add_property(@arg1);
        $add_property(@arg2);
      else
        add_property(@arg1);
        add_property(@arg2);
      endif
      $options["ansi"]:add_name(args[1]);
      return 1;
    endif
  endverb

  verb "add_code" (this none this) owner: #36 flags: "rxd"
    ":add_code (STR code, NUM/STR sequence, STR group)";
    "Adds a new code, <code> and adds it to group <group>.";
    "If <sequence> is a string, it is used as the ANSI sequence, otherwise";
    "it uses 'e[<sequence>m'.  'e' is replaced with the escape character, and";
    "'b' is replaced with the beep character in <sequence>";
    if (!this:trusts(caller_perms()))
      return E_PERM;
    elseif (length(args) < 3)
      return E_ARGS;
    elseif (!(args[1] && typeof(args[1]) == STR && !$object_utils:has_property(this, cn = tostr("code_", args[1])) && (group = args[3]) in {@this.groups, E_NONE}))
      return E_INVARG;
    else
      code = args[2];
      if (typeof(code) == NUM)
        code = tostr("e[", code, "m");
      endif
      arg = {this, cn, code, {$code_utils:verb_perms(), "r"}};
      if ($object_utils:has_verb(#0, "add_property"))
        $add_property(@arg);
      else
        add_property(@arg);
      endif
      if (args[3] == E_NONE)
        this.extra_codes = setadd(this.extra_codes, args[1]);
      else
        this.("group_" + args[3]) = setadd(this.("group_" + args[3]), args[1]);
      endif
      this:update_all();
      return 1;
    endif
  endverb

  verb "show_who_listing" (this none this) owner: #36 flags: "rx"
    ":show_who_listing(players[,more_players])";
    " prints a listing of the indicated players.";
    " For players in the first list, idle/connected times are shown if the player is logged in, otherwise the last_disconnect_time is shown.  For players in the second list, last_disconnect_time is shown, no matter whether the player is logged in.";
    idles = itimes = offs = otimes = listing = {};
    for p in (args[2])
      if (!valid(p))
        listing = {@listing, tostr(p, " <invalid>")};
      elseif (typeof(t = p.last_disconnect_time) == NUM)
        p in offs || ((offs = {@offs, p}) && (otimes = {@otimes, {-t, -t, p}}));
      elseif (is_player(p))
        listing = {@listing, tostr(p.name, " (", p, ") ", t == E_PROPNF ? "is not a $player." | "has a garbled .last_disconnect_time.")};
      else
        listing = {@listing, tostr(p.name, " (", p, ") is not a player.")};
      endif
    endfor
    for p in (args[1])
      if (p in offs)
      elseif (!valid(p))
        listing = {@listing, tostr(p, " <invalid>")};
      elseif (typeof(i = idle_seconds(p)) != ERR && p in connected_players())
        p in idles || ((idles = {@idles, p}) && (itimes = {@itimes, {i, connected_seconds(p), p}}));
      elseif (typeof(t = p.last_disconnect_time) == NUM)
        (offs = {@offs, p}) && (otimes = {@otimes, {-t, -t, p}});
      elseif (is_player(p))
        listing = {@listing, tostr(p.name, " (", p, ") not logged in.", t == E_PROPNF ? "  Not a $player." | "  Garbled .last_disconnect_time.")};
      else
        listing = {@listing, tostr(p.name, " (", p, ") is not a player.")};
      endif
    endfor
    if (!(idles || offs))
      return 0;
    endif
    idles = $list_utils:sort_alist(itimes);
    offs = $list_utils:sort_alist(otimes);
    headers = {"Player name", @idles ? {"Connected", "Idle time"} | {"Last disconnect time", ""}, "Location"};
    total_width = caller:linelen() || 79;
    max_name = total_width / 4;
    name_width = length(headers[1]);
    names = locations = {};
    for lst in ({@idles, @offs})
      ticks_left() < 2000 || seconds_left() < 2 && suspend(0);
      p = lst[3];
      "p.name and this:ansi_title(p) should be the same length, saves a call to this:length";
      namestr = tostr(this:cutoff(this:ansi_title(p), 1, min(max_name, z = length(p.name)), 1), " (", p, ")");
      name_width = max(z + 3 + length(tostr(p)), name_width);
      names = {@names, namestr};
      typeof(wlm = p.location:who_location_msg(p)) == STR || (wlm = valid(p.location) ? p.location.name | tostr("** Nowhere ** (", p.location, ")"));
      locations = {@locations, wlm};
    endfor
    time_width = offs ? 15 | 13;
    before = {0, w1 = 3 + name_width, w2 = w1 + time_width, w2 + time_width};
    su = $string_utils;
    tell1 = headers[1];
    tell2 = su:space(tell1, "-");
    for j in [2..4]
      tell1 = su:left(tell1, before[j]) + headers[j];
      tell2 = su:left(tell2, before[j]) + su:space(headers[j], "-");
    endfor
    listing = {@listing, tell1[1..min(length(tell1), total_width)]};
    listing = {@listing, tell2[1..min(length(tell2), total_width)]};
    "...";
    "...print lines...";
    "...";
    active = 0;
    for i in [1..total = (ilen = length(idles)) + length(offs)]
      if (i <= ilen)
        lst = idles[i];
        if (lst[1] < 5 * 60)
          active = active + 1;
        endif
        l = {names[i], su:from_seconds(lst[2]), su:from_seconds(lst[1]), locations[i]};
      else
        lct = offs[i - ilen][3].last_connect_time;
        ldt = offs[i - ilen][3].last_disconnect_time;
        ctime = caller:ctime(ldt) || ctime(ldt);
        l = {names[i], lct <= time() ? ctime | "Never", "", locations[i]};
        if (i == ilen + 1 && idles)
          listing = {@listing, su:space(before[2]) + "------- Disconnected -------"};
        endif
      endif
      tell1 = l[1];
      for j in [2..4]
        tell1 = su:left(tell1, before[j]) + l[j];
      endfor
      listing = {@listing, this:cutoff(tell1, 1, min(this:length(tell1), total_width))};
      if ($command_utils:running_out_of_time())
        if ($login:is_lagging())
          "Check lag two ways---global lag, but we might still fail due to individual lag of the queue this runs in, so check again later.";
          listing = {@listing, tostr("Plus ", total - i, " other players (", total, " total; out of time and lag is high).")};
          return;
        endif
        now = time();
        suspend(0);
        if (time() - now > 10)
          listing = {@listing, tostr("Plus ", total - i, " other players (", total, " total; out of time and lag is high).")};
          return;
        endif
      endif
    endfor
    "...";
    "...epilogue...";
    listing = {@listing, ""};
    if (total == 1)
      active_str = ", who has" + (active == 1 ? "" | " not");
    else
      if (active == total)
        active_str = active == 2 ? "s, both" | "s, all";
      elseif (active == 0)
        active_str = "s, none";
      else
        active_str = tostr("s, ", active);
      endif
      active_str = tostr(active_str, " of whom ha", active == 1 ? "s" | "ve");
    endif
    listing = {@listing, tostr("Total: ", total, " player", active_str, " been active recently.")};
    vrb = caller == $login || $perm_utils:controls($code_utils:verb_perms(), caller) ? "notify" | "tell";
    for line in (listing)
      caller:(vrb)(line);
      seconds_left() < 2 || ticks_left() < 4000 && suspend(0);
    endfor
    return total;
  endverb

  verb "notify" (this none this) owner: #2 flags: "rx"
    ":notify (OBJ player, STR line[, extra parameters for notify])";
    set_task_perms(caller_perms());
    {plr, line, @extra} = args;
    "...use property_info() instead of $object_utils:isa to save ticks...";
    if (index(line, "[") && valid(plr) && property_info(plr, "ansi_options") && this.active && !(task_id() in this.noansi_queue) && !plr:ansi_option("ignore"))
      codes = typeof(z = plr.replace_codes) == NUM ? this.replace_code_pointers[z] | z;
      esc = plr:ansi_option("escape");
      "... save more ticks here by using 'in' instead of 'ansi_option'...";
      truecolor_enabled = "truecolor" in plr.ansi_options;
      xterm_256 = "256" in plr.ansi_options;
      backgrounds = "backgrounds" in plr.ansi_options;
      while (m = match(line, this.notify_regexp))
        z = line[m[1] + 1..m[2] - 1];
        if (z in codes)
          code = this:get_code(z, esc);
        elseif (!backgrounds && z[1..2] == "b:")
          code = "";
        elseif (z[1..2] == "b:" && z[3..$] in codes)
          code = this:get_code(z, esc);
        elseif (z == "random")
          code = this:get_code(this.random_colors[random(length(this.random_colors))], esc);
        elseif (z == ":random" && xterm_256)
          code = this:get_code(tostr(":", random(255)), esc, 0, 1);
        elseif (xterm_256 && (z[1] == ":" || z[1..3] == "b::"))
          code = this:get_code(z, esc, 0, 1);
        elseif (truecolor_enabled && index(z, ":"))
          code = this:get_code(z, esc, m);
        else
          code = "";
        endif
        line[m[1]..m[2]] = code;
      endwhile
      line = strsub(line, "[null]", "");
    endif
    return notify(plr, line, @extra);
  endverb

  verb "add_noansi" (this none this) owner: #36 flags: "rxd"
    ":add_noansi()";
    "Called by tasks to tell players to ignore any ANSI codes from them.";
    "Can be undone with a call to :remove_noansi";
    if (length(this.noansi_queue) > 30 && !$code_utils:task_valid(this.noansi_task))
      fork tid (0)
        this:cleanup_noansi();
      endfork
      this.noansi_task = tid;
    endif
    this.noansi_queue = setadd(this.noansi_queue, task_id());
  endverb

  verb "remove_noansi" (this none this) owner: #36 flags: "rxd"
    ":remove_noansi()";
    "Start translating the ANSI codes from the current task again";
    this.noansi_queue = setremove(this.noansi_queue, task_id());
  endverb

  verb "tell" (this none this) owner: #36 flags: "rxd"
    return;
  endverb

  verb "self_diagnostic" (this none this) owner: #36 flags: "rx"
    ":self_diagnostic ([NUM fix[, OBJ plyr]]) => NUM errors fixed";
    "Reports all errors found to <plyr> or the current player.";
    "Fixes any errors it can if <fix> is specified and true.";
    "<errors fixed> is the errors that could have been fixed if <fix> is false.";
    if (!this:trusts(caller_perms()))
      return E_PERM;
    else
      count = 0;
      for x in (this.diagnostic_tests)
        player:tell("Running test \"", x, "\"...");
        count = count + !!this:("test_" + x)(@args);
      endfor
      return count;
    endif
  endverb

  verb "trusts" (this none this) owner: #36 flags: "rxd"
    ":trusts (OBJ player) => true of <player> is trusted by the ANSI system.";
    return args[1].wizard || args[1] == this.owner || args[1] in this.trusted;
  endverb

  verb "cutoff_locs" (this none this) owner: #36 flags: "rx"
    ":cutoff_locs (STR string,NUM start,NUM end[,NUM extra][, NUM suspendok])";
    "                                                       => {nstart, nend}";
    "Takes <start> and <end>, fixes them to compensate for the ANSI codes, and";
    "returns them.  If <extra> is provided and true, <nend> will include the";
    "codes after the ending letter.";
    start = args[2];
    end = args[3];
    if (typeof(string = args[1]) != STR)
      return E_INVARG;
    elseif (!(index(string, "[") && this.active))
      return {start, end == "$" ? length(string) | end};
    elseif (start > end)
      return args[2..3];
    endif
    i = begin = 0;
    x = 1;
    extra = {@args, 0}[4];
    reg = this.all_regexp;
    l = length(string);
    suspendok = {@args, 0}[5];
    while (x <= l)
      suspendok && (ticks_left() < 1000 || seconds_left() < 2) && player:tell("suspending...") && suspend(0);
      if (m = match(string, reg))
        i = i + (m[1] - 1);
        if (!begin && i + 1 >= start)
          begin = x + m[1] - i + start - 2;
          if (end == "$")
            return {begin, l};
          endif
        endif
        if (begin && i - extra >= end)
          return {begin, x + m[1] - i + end - 2};
        endif
        x = x + m[2];
        string[1..m[2]] = "";
      else
        return {begin || x - i + start - 1, end == "$" ? l | x - i + end - 1};
      endif
    endwhile
    return end == i && begin ? {begin, l} | E_RANGE;
  endverb

  verb "cleanup_noansi" (this none this) owner: #36 flags: "rxd"
    while (this.noansi_queue && !$command_utils:running_out_of_time())
      x = this.noansi_queue[1];
      if (!$code_utils:task_valid(x))
        this.noansi_queue = setremove(this.noansi_queue, x);
      endif
    endwhile
  endverb

  verb "test_benchmark" (this none this) owner: #36 flags: "rxd"
    if (caller != this)
      return E_PERM;
    else
      new = $recycler:_create($ansi_pc);
      if (typeof(new) != OBJ)
        return player:tell("Unable to create Benchmark test player: ", new);
      endif
      new:set_name("Benchmark_test_player");
      suspend(0);
      ticks = ticks_left();
      seconds = seconds_left();
      for x in [1..3]
        $ansi_utils:notify(new, "[blue]B[bold]e[unbold]n[bold]c[unbold]h[bold]m[unbold]a[bold]r[unbold]k [red]T[bold]e[unbold]s[bold]t [random].[random].[random].[random].[random].");
      endfor
      for x in [1..3]
        $ansi_utils:notify(new, "[123:123:123]B[1:1:1]e[0:100:0]n[100:0:100]c[100:100:100]h[100:0:0]m[0:0:100]a[255:0:255]r[255:0:0]k [0:255:0]T[0:0:255]e[255:255:0]s[123:45:6]t.");
      endfor
      ticks = ticks - ticks_left();
      seconds = seconds - seconds_left();
      new:set_ansi_option("colors", 1);
      new:set_ansi_option("escape", "~");
      new:set_ansi_option("misc", 1);
      ticks = ticks + ticks_left();
      seconds = seconds + seconds_left();
      for x in [1..3]
        $ansi_utils:notify(new, "[blue]B[bold]e[unbold]n[bold]c[unbold]h[bold]m[unbold]a[bold]r[unbold]k [red]T[bold]e[unbold]s[bold]t [random].[random].[random].[random].[random].");
      endfor
      for x in [1..3]
        $ansi_utils:notify(new, "[123:123:123]B[1:1:1]e[0:100:0]n[100:0:100]c[100:100:100]h[100:0:0]m[0:0:100]a[255:0:255]r[255:0:0]k [0:255:0]T[0:0:255]e[255:255:0]s[123:45:6]t.");
      endfor
      for x in [1..3]
        $ansi_utils:notify(new, "Testing...");
      endfor
      this:add_noansi();
      for x in [1..3]
        $ansi_utils:notify(new, "[blue]B[bold]e[unbold]n[bold]c[unbold]h[bold]m[unbold]a[bold]r[unbold]k [red]T[bold]e[unbold]s[bold]t [random].[random].[random].[random].[random].");
      endfor
      for x in [1..3]
        $ansi_utils:notify(new, "[123:123:123]B[1:1:1]e[0:100:0]n[100:0:100]c[100:100:100]h[100:0:0]m[0:0:100]a[255:0:255]r[255:0:0]k [0:255:0]T[0:0:255]e[255:255:0]s[123:45:6]t.");
      endfor
      this:remove_noansi();
      ticks = ticks - ticks_left();
      seconds = seconds - seconds_left();
      $recycler:_recycle(new);
      player:tell("21 notifies: ", ticks, " tick", ticks == 1 ? "" | "s", ", ", seconds, " second", seconds == 1 ? "" | "s", ".");
    endif
  endverb

  verb "cutoff_assign" (this none this) owner: #36 flags: "rxd"
    ":cutoff_assign (STR string, NUM start, NUM end, STR replacement[, NUM extra])";
    "                                => STR";
    "Example:";
    "  string[2..3] = \"test\";";
    "Is the same as:";
    "  string = $ansi_utils:cutoff_assign(string, 2, 3, \"test\");";
    "Except that it ignores the ANSI codes in <string> when finding <start> and";
    "<end>.  If <extra> is specified and true, any codes after <end> but before";
    "the next character will also be overwritten.";
    if (typeof(a = this:cutoff_locs(@listdelete(args, 4))) == LIST)
      args[1][a[1]..a[2]] = args[4];
      return args[1];
    else
      return a;
    endif
  endverb

  verb "setadd" (this none this) owner: #36 flags: "rxd"
    ":setadd (LIST l, value) => LIST";
    "Does the same thing as the built-in setadd(), but if <value> is a string,";
    "it won't be added to <l> if <value> with it's ANSI codes stripped out equals";
    "any of <l>'s elements with their ANSI codes stripped out.";
    l = args[1];
    if (typeof(value = args[2]) == STR && this:contains_codes(value))
      nvalue = this:delete(value);
      for x in (l)
        if (typeof(x) == STR && this:delete(x) == nvalue)
          return l;
        endif
      endfor
    endif
    return setadd(l, value);
  endverb

  verb "setremove" (this none this) owner: #36 flags: "rxd"
    ":setremove (LIST l, value) => LIST";
    "Does the same thing as the built-in setremove(), but if <value> is a";
    "string, it will remove any string in <l> that, when it's ANSI codes are";
    "stripped out, is equal to <value> with it's ANSI codes stripped out.";
    l = args[1];
    if (typeof(value = args[2]) != STR || !this:contains_codes(value))
      return setremove(l, value);
    endif
    nvalue = this:delete(value);
    for x in [-length(l)..-1]
      x = -x;
      if (typeof(l[x]) == STR && this:delete(l[x]) == nvalue)
        l = listdelete(l, x);
      endif
    endfor
    return l;
  endverb

  verb "ansi_status" (this none this) owner: #36 flags: "rxd"
    mess = {};
    mess = {@mess, tostr("ANSI Version ", this.version, ":")};
    a = 0;
    for x in (this.groups)
      a = a + length(this.("group_" + x));
    endfor
    mess = {@mess, tostr("It is ", this.active ? "currently" | "not", " active.  There are ", $string_utils:english_number(a), " codes defined in ", $string_utils:english_number(length(this.groups)), " groups.  There ", length(this.noansi_queue) == 1 ? "is" | "are", " ", $string_utils:english_number(length(this.noansi_queue)), " tasks in the ignore ANSI task queue, and the cleanup task is ", $code_utils:task_valid(this.noansi_queue) ? "currently" | "not", " running.")};
    return mess;
  endverb

  verb "terminate_normal" (this none this) owner: #36 flags: "rxd"
    ":terminate_normal (STR string) => STR <string> with a [[null]normal] code";
    "tacked onto the end if there wasn't one";
    if (!index(string = args[1], "["))
      return string;
    endif
    m = rmatch(string, this.terminate_regexp);
    while (string && m && m[2] == length(string))
      string = string[1..m[1] - 1];
      m = rmatch(string, this.terminate_regexp);
    endwhile
    return string && string + (m && string[m[1]..m[2]] != "[normal]" ? "[normal]" | "");
  endverb

  verb "left" (this none this) owner: #36 flags: "rxd"
    "$ansi_utils:left(string,width[,filler])";
    "";
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> followed by enough filler to make it that wide. If <width> is negative and the length of <string> is greater than the absolute value of <width>, then the <string> is cut off at <width>.";
    "";
    "The <filler> is optional and defaults to \" \"; it controls what is used to fill the resulting string when it is too short.  The <filler> is replicated as many times as is necessary to fill the space in question.";
    return this:terminate_normal(z = (l = this:length(out = tostr(args[1]))) < (len = abs(args[2])) ? out + this:space(l - len, length(args) >= 3 && args[3] || " ") | (args[2] > 0 ? out | this:cutoff(out, 1, len)));
  endverb

  verb "right" (this none this) owner: #36 flags: "rxd"
    "$ansi_utils:right(string,width[,filler])";
    "";
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> preceded by enough filler to make it that wide. If <width> is negative and the length of <string> is greater than the absolute value of <width>, then <string> is cut off at <width>.";
    "";
    "The <filler> is optional and defaults to \" \"; it controls what is used to fill the resulting string when it is too short.  The <filler> is replicated as many times as is necessary to fill the space in question.";
    return this:terminate_normal((l = this:length(out = tostr(args[1]))) < (len = abs(args[2])) ? this:space(len - l, length(args) >= 3 && args[3] || " ") + out | (args[2] > 0 ? out | this:cutoff(out, 1, len)));
  endverb

  verb "centre center" (this none this) owner: #36 flags: "rxd"
    "$ansi_utils:center(string,width[,lfiller[,rfiller]])";
    "";
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> preceded and followed by enough filler to make it that wide.  If <width> is negative and the length of <string> is greater than the absolute value of <width>, then the <string> is cut off at <width>.";
    "";
    "The <lfiller> is optional and defaults to \" \"; it controls what is used to fill the left part of the resulting string when it is too short.  The <rfiller> is optional and defaults to the value of <lfiller>; it controls what is used to fill the right part of the resulting string when it is too short.  In both cases, the filler is replicated as many times as is necessary to fill the space in question.";
    return this:terminate_normal((l = this:length(out = tostr(args[1]))) < (len = abs(args[2])) ? tostr(this:space((len - l) / 2, lfill = length(args) >= 3 && args[3] || " "), out, this:space((len - l + 1) / -2, length(args) >= 4 ? args[4] | lfill)) | (args[2] > 0 ? out | this:cutoff(out, 1, len)));
  endverb

  verb "columnize columnise" (this none this) owner: #36 flags: "rxd"
    "columnize (items, n [, width]) - Turn a one-column list of items into an n-column list. 'width' is the last character position that may be occupied; it defaults to a standard screen width. Example: To tell the player a list of numbers in three columns, do 'player:tell_lines ($string_utils:columnize ({1, 2, 3, 4, 5, 6, 7}, 3));'.";
    items = args[1];
    n = args[2];
    width = {@args, 79}[3];
    height = (length(items) + n - 1) / n;
    items = {@items, @$list_utils:make(height * n - length(items), "")};
    colwidths = {};
    for col in [1..n - 1]
      colwidths = listappend(colwidths, 1 - (width + 1) * col / n);
    endfor
    result = {};
    for row in [1..height]
      line = tostr(items[row]);
      for col in [1..n - 1]
        line = tostr(this:terminate_normal(this:left(line, colwidths[col])), " ", items[row + col * height]);
      endfor
      result = listappend(result, this:terminate_normal(this:cutoff(line, 1, min(this:length(line), width))));
    endfor
    return result;
  endverb

  verb "space" (this none this) owner: #36 flags: "rxd"
    "space(len,fill) returns a string of length abs(len) consisting of copies of fill.  If len is negative, fill is anchored on the right instead of the left.";
    n = args[1];
    typeof(n) == STR && (n = this:length(n));
    if (" " != (fill = {@args, " "}[2]))
      fill = fill + fill;
      fill = fill + fill;
      fill = fill + fill;
    elseif ((n = abs(n)) < 70)
      return "                                                                      "[1..n];
    else
      fill = "                                                                      ";
    endif
    m = (n - 1) / this:length(fill);
    while (m)
      fill = fill + fill;
      m = m / 2;
    endwhile
    return n > 0 ? this:cutoff(fill, 1, n) | this:cutoff(fill, (f = this:length(fill)) + 1 + n, f);
  endverb

  verb "ansi_title" (this none this) owner: #36 flags: "rx"
    ":ansi_title (OBJ player[, STR name]) => STR <player>'s title";
    "If <name> is specified, it will be used instead of <player>.name";
    name = {@args, args[1].name}[2];
    for x in (args[1].ansi_title)
      if (typeof(x[2]) == LIST)
        nn = x[2][random(length(x[2]))];
      else
        nn = x[2];
      endif
      nn && (name = strsub(name, x[1], nn));
    endfor
    return name;
  endverb

  verb "strip_black" (this none this) owner: #36 flags: "rxd"
    gray = 0;
    x = 1;
    string = args[1];
    l = length(string);
    while (x <= l && (m = match(string[x..l], this.all_regexp)))
      code = string[x + m[1]..x + m[2] - 2];
      if (code in {"gray", "grey"})
        gray = 1;
      elseif (code in this.group_colors)
        gray = 0;
      elseif (gray && code == "unbold")
        string[x + m[2]..x + m[2] - 1] = "[white]";
        x = x + 7;
      endif
      x = x + m[2];
    endwhile
    return string;
  endverb

  verb "quote_ansi" (this none this) owner: #36 flags: "rxd"
    ":quote_ansi (STR string) => STR new_string";
    "Puts a [[null]null] code in the middle of all of the other codes in <string>";
    "so they won't be replaced.";
    return strsub(args[1], "[", "[[null]");
    "...should probably only fix real codes, but this works for now...";
  endverb

  verb "update_player_codes" (this none this) owner: #36 flags: "rxd"
    ":update_player_codes (OBJ player)";
    "Updates <player>'s .replace_codes property";
    if (!this:trusts(caller_perms()))
      return E_PERM;
    elseif ($object_utils:isa(plr = args[1], $ansi_pc))
      codes = {};
      for x in (this.groups)
        if (plr:ansi_option(x))
          codes = {@codes, @this.("group_" + x)};
          x == "extra" || (codes = setadd(codes, "normal"));
        endif
      endfor
      return plr.replace_codes = codes in this.replace_code_pointers || codes;
    endif
  endverb

  verb "fill_string" (this none this) owner: #36 flags: "rxd"
    "fill(string,width[,prefix])";
    "tries to cut <string> into substrings of length < <width> along word boundaries.  Prefix, if supplied, will be prefixed to the 2nd..last substrings.";
    if (length(args) < 2)
      width = 2 + player:linelen();
      prefix = "";
    else
      width = args[2] + 1;
      prefix = {@args, ""}[3];
    endif
    if (width < 3 + length(prefix))
      return E_INVARG;
    endif
    string = "$" + args[1] + " $";
    len = this:length(string);
    if (len <= width)
      last = len - 1;
      next = len;
    else
      last = this:rindex(this:cutoff(string, 1, width), " ");
      if (last < (width + 1) / 2)
        last = width + this:index(this:cutoff(string, width + 1, "$", 1), " ");
      endif
      next = last;
      while (string[next = next + 1] == " ")
      endwhile
    endif
    while (string[last = last - 1] == " ")
    endwhile
    ret = {this:cutoff(string, 2, last)};
    width = width - length(prefix);
    minlast = (width + 1) / 2;
    while (next < len)
      string = this:cutoff_assign(string, 1, next - 1, "$");
      "string = \"$\" + string[next..len];";
      len = len - next + 2;
      if (len <= width)
        last = len - 1;
        next = len;
      else
        last = this:rindex(this:cutoff(string, 1, width), " ");
        if (last < minlast)
          last = width + this:index(this:cutoff(string, width + 1, "$", 1), " ");
        endif
        next = last;
        while (string[next = next + 1] == " ")
        endwhile
      endif
      while (string[last = last - 1] == " ")
      endwhile
      if (last > 1)
        ret = {@ret, prefix + this:cutoff(string, 2, last)};
      endif
    endwhile
    return ret;
  endverb

  verb "@ansi-setup" (this none none) owner: #2 flags: "rx"
    "Usage:  @ansi-setup <this>";
    "Used to fix various core utilities to work with ANSI. This verb can only be used by a wizard, and needs wizperms to run.";
    "Ugh, this verb is getting out of control, this stuff should all be moved to diagnostic tests.";
    if (!player.wizard)
      player:tell("This verb was intended to fix up the rest of a MOO's core so it can function properly with the ANSI PC. If something's wrong, ask a wizard to set this up for you.");
    elseif (!$code_utils:verb_perms().wizard)
      player:tell("This verb needs to be wizpermed before it can work.");
    elseif (!$command_utils:yes_or_no("This will change various verbs in the core so they can be used with the ANSI PC, overwriting the previous verbs. Are you sure you want to do this?"))
      player:notify("Well, okay then.");
    else
      set_task_perms(valid(cp = caller_perms()) ? cp | player);
      spiffy = 1;
      "----== Corify Objects ==----";
      for x in ({"help", "pc", "utils", "options"})
        prop = "ansi_" + x;
        if (!$object_utils:has_property($sysobj, prop))
          add_property($sysobj, prop, #-1, {player, "r"});
          player:notify(tostr("Creating a $", prop, " property."));
        endif
        if (valid($sysobj.(prop)))
        elseif (x == "utils")
          player:notify(tostr("Setting $", prop, " to ", $string_utils:nn(this), "."));
          $sysobj.(prop) = this;
        else
          objects = {};
          for o in ({@player.owned_objects || {}, @player.public_identity.owned_objects || {}, @player.contents || {}, @player.location.contents || {}})
            if (index(o.name, "ANSI") && index(o.name, strsub(x, "s", "")))
              objects = setadd(objects, o);
            endif
          endfor
          if (!objects)
            return player:notify(tostr("Unable to find $", prop, ", please port this object and set #0.", prop, " to it's object number. If you created a new player to own ANSI objects, you might want to temporarily set your .public_identity property to that player and rerun this verb. Remember to reset the property when you're done."));
          elseif (length(objects) == 1)
            player:notify(tostr("Setting $", prop, " to ", $string_utils:nn(objects[1]), "."));
            $sysobj.(prop) = objects[1];
          else
            return player:notify(tostr("Found ", length(objects), " objects that could be $", prop, ", please set #0.", prop, " to the object number of the correct one:  ", $string_utils:nn_list(objects)));
          endif
        endif
      endfor
      "----== Wizperm everything that should be ==----";
      for x in (this.need_wizperms)
        if (!(info = verb_info(y = $sysobj.(x[1]), x[2]))[1].wizard)
          player:notify(tostr("Wizperming $", x[1], ":", x[2], "..."));
          set_verb_info(y, x[2], listset(info, player, 1));
        endif
      endfor
      if (!($ansi_help in (typeof(ah = $ansi_pc.help) == LIST ? ah | {ah})))
        player:notify("Setting $ansi_pc.help...");
        $ansi_pc.help = $ansi_help;
      endif
      "----== Various core hacks ==----";
      su = $string_utils;
      if (!$object_utils:has_callable_verb(su, "redirect_ansi"))
        player:notify("Adding $string_utils:redirect_ansi...");
        add_verb(su, {$hacker, "rx", "redirect_ansi"}, {"this", "none", "this"});
        set_verb_code(su, "redirect_ansi", this.redirect_su_code);
      endif
      for x in (this.redirect_su_names)
        if ((info = verb_info(su, x)) && !index(info[3], "redirect_ansi"))
          nn = "";
          for y in ($string_utils:explode(info[3], " "))
            nn = tostr(nn, " ", y, "(noansi)");
          endfor
          set_verb_info(su, x, listset(info, $string_utils:triml(nn), 3));
          player:notify(tostr("Renaming $string_utils:\"", info[3], "\" to \"", nn, "\"..."));
        endif
      endfor
      redirect = $string_utils:from_list({"redirect_ansi", @this.redirect_su_names}, " ");
      if ((info = verb_info(su, "redirect_ansi"))[3] != redirect)
        player:notify(tostr("Renaming $string_utils:redirect_ansi to \"", redirect, "\"."));
        set_verb_info(su, "redirect_ansi", listset(info, redirect, 3));
      endif
      !(length(vc = verb_code($login, "notify")) == 2 && index(vc[2], "$ansi_utils:delete")) && $command_utils:yes_or_no("Update $login:notify?") ? set_verb_code($login, "notify", {"(caller!=$ansi_utils)&&set_task_perms(caller_perms());notify(player,$ansi_utils:delete(args[1]));"}) || player:notify("$login:notify changed.") | (spiffy = 0) || player:notify("$login:notify left alone.");
      thatline = "line[1..min(width, length(line))]";
      newline = "$ansi_utils:cutoff(line,1,min(width,$ansi_utils:length(line)))";
      for x in ({"mail_agent", "big_mail_recipient"})
        vc = $string_utils:print(verb_code(y = $sysobj.(x), vn = "display_seq_headers"));
        player:notify(tostr("$", x, ":", vn, " ", index(vc, thatline) && $command_utils:yes_or_no(tostr("Replace \"", thatline, "\" in $", x, ":", vn, " with \"", newline, "\"?")) && set_verb_code(y, vn, $string_utils:to_value(strsub(vc, thatline, newline))[2]) == {} ? "changed." | (spiffy = 0) || "left alone."));
      endfor
      code = {};
      for x in (verb_code($login, "@who"))
        code = {@code, strsub(x, "$code_utils:show_who_listing", "$ansi_utils:show_who_listing")};
      endfor
      if (code != verb_code($login, "@who"))
        player:notify("Setting $login:@who...");
        set_verb_code($login, "@who", code);
      endif
      code = {};
      for x in (verb_code($guest, "do_reset", 0, 0))
        if ((m = match(x, "^for x in (%(%{.+%}%))$")) && (info = $string_utils:to_value(substitute("%1", m)))[1])
          x = tostr("for x in (", $string_utils:print($set_utils:union(info[2], $ansi_utils.reset_guest_props)), ")");
        endif
        code = {@code, x};
      endfor
      if (code != verb_code($guest, "do_reset", 0, 0))
        player:notify("Setting $guest:do_reset...");
        set_verb_code($guest, "do_reset", code);
      endif
      code = {};
      for x in (verb_code($prog, "@list", 0, 0))
        if (code == 0)
        elseif (index(x, "$ansi_utils:add_noansi("))
          code = 0;
        elseif (index(x, "player:notify(tostr(what, \":\", fullname, \"") == 1)
          code = {@code, "$ansi_utils:add_noansi();", x};
        else
          code = {@code, x};
        endif
      endfor
      if (code && code != verb_code($prog, "@list", 0, 0))
        player:notify("Setting $prog:@list...");
        set_verb_code($prog, "@list", code);
      endif
      code = {};
      a = {0, 0};
      for x in (verb_code($generic_editor, "list_line"))
        a[1] = a[1] || index(x, "$ansi_utils:add_noansi(");
        a[2] = a[2] || index(x, "$ansi_utils:remove_noansi(");
        code = {@code, x};
      endfor
      a[1] || (code = {"$ansi_utils:add_noansi();", @code});
      a[2] || (code = {@code, "$ansi_utils:remove_noansi();"});
      if (code != verb_code($generic_editor, "list_line"))
        player:notify("Setting $generic_editor:list_line...");
        set_verb_code($generic_editor, "list_line", code);
      endif
      if (!$object_utils:defines_verb($player_db, "insert"))
        player:notify("Adding $player_db:insert...");
        add_verb($player_db, {player, "rxd", "insert"}, {"this", "none", "this"});
        set_verb_code($player_db, "insert", this.plr_db_insert);
      elseif (verb_code($player_db, "insert") != this.plr_db_insert)
        player:notify("$player_db:insert already exists, you will have to edit it manually for the ANSI, see $ansi_utils.plr_db_insert for sample code.");
      endif
      if (verb_code($generic_editor, "fill_string") != this.ge_fill_string)
        a = "";
        while ($object_utils:has_verb($generic_editor, vname = tostr("fill_string(noansi", a, ")")))
          a = toint(a) + 1;
        endwhile
        set_verb_info($generic_editor, "fill_string", listset(verb_info($generic_editor, "fill_string"), vname, 3));
        add_verb($generic_editor, {player, "rx", "fill_string"}, {"this", "none", "this"}) || player:notify("Adding new $generic_editor:fill_string...");
        set_verb_code($generic_editor, "fill_string", this.ge_fill_string);
      endif
      "...ugh, I want to put $prog:@dump in here but the one I put on NestMOO is too big to search...";
      "----== Set the non-printable characters ==----";
      for x in ({{"escape", 27, "033"}, {"beep", 7, "007"}})
        chr = x[1];
        code = x[2];
        octal = x[3];
        if (typeof(this.(chr)) != STR || length(this.(chr)) != 1)
          if (eval(";return chr(64);")[1])
            player:notify(tostr("Setting $ansi_utils.", chr, " with chr()..."));
            eval(tostr(";$ansi_utils.(\"", chr, "\")=chr(", code, ");"));
          elseif ((eval = eval(";return filelist(\"\", \"\");"))[1] || (eval = eval(";return file_list(\"\");"))[1])
            files = typeof(eval[2][1]) == LIST ? eval[2][1] | eval[2];
            if (chr + ".chr" in files)
              player:notify(tostr("Setting $ansi_utils.", chr, " from file ", chr, ".chr..."));
              if (typeof(eval[2][1]) == LIST)
                "Setup for FUP";
                eval(tostr(";$ansi_utils.(\"", chr, "\")=fileread(\"\", \"", chr, ".chr\")[1];"));
              else
                "Setup for FIO";
                handle = file_open(tostr("/", chr, ".chr"), "r-tn");
                $ansi_utils.(chr) = file_readline(handle);
                file_close(handle);
              endif
            else
              player:notify(tostr("File builtin detected, please create a file named \"", chr, ".chr\" in the files directory and put an ASCII character ", code, " in it.  This can be done on most systems with the command:  echo -e '\\", octal, "' > ", chr, ".chr  from the files directory."));
              spiffy = 0;
            endif
          else
            z = this.(chr) = tostr("<----- ", $string_utils:uppercase(chr), " ----->");
            player:notify(tostr("I can't find any way to set $ansi_utils.", chr, ", please either install the FUP, FileIO, or chr() server patches and rerun this verb, or shut down the MOO, load the DB into an editor, and replace \"", z, "\" with an ASCII character ", code, "."));
            spiffy = 0;
          endif
        endif
      endfor
      if (this.active)
        player:notify("@ansi-setup finished.");
      elseif (!spiffy)
        player:notify("@ansi-setup can not verify that everything has been set up correctly, you will probably have to rerun this verb.  If you're sure everything is correct, you can type:  ;;$ansi_utils.active=1;  to activate it.");
      elseif ($command_utils:yes_or_no("Everything seems to be set up correctly, activate the ANSI system?"))
        this.active = 1;
        "...raw notify() the first message in case it breaks, we're wizpermed anyway...";
        notify(player, "The ANSI system is now active, it can be deactivated by typing: ;;$ansi_utils.active = 0;");
        player:notify(tostr("Welcome to ANSI version ", this.version, "."));
      else
        player:notify("Not activating the ANSI system, you can do this manually by typing: ;;$ansi_utils.active = 1;  when you're sure everything's set up correctly.");
      endif
    endif
  endverb

  verb "approximate_256" (this none this) owner: #36 flags: "rxd"
    "Attempt to downscale a 24-bit RGB color into an 8-bit 256 color.";
    "Disclaimer: Looks terrible.";
    {r, g, b} = args;

    "Scale channels down to fit in fewer bits";
    red = toint(r) * 8 / 256;
    green = toint(g) * 8 / 256;
    blue = toint(b) * 4 / 256;

    "Combine channels into a single 8-bit value";
    ret = red * 32 + green * 4 + blue;

    return ret;
  endverb

  verb "color_selector" (this none this) owner: #36 flags: "rxd"
    {?raw = 0, ?input = 0, ?foreground_color = 0} = args;
    colors = {"Red", "Green", "Blue", "Yellow", "Cyan", "Purple", "Gray", "White"};
    backgrounds = {"Red", "Green", "Blue", "Yellow", "Cyan", "Purple", "White"};
    menu = codes = {};
    for x in (colors)
      if (!foreground_color)
        menu = {@menu, tostr(this:hr_to_code(code = x), x, "[normal]")};
        codes = {@codes, code};
        menu = {@menu, tostr(this:hr_to_code(code = "bold|" + x), "Bold ", x, "[normal]")};
        codes = {@codes, code};
      endif
      if (x in backgrounds)
        menu = {@menu, tostr(this:hr_to_code(code = "b:" + x), x == "white" ? "[gray]" | (foreground_color ? this:Hr_to_code(foreground_color) | "[white]"), "Background ", x, "[normal]")};
        codes = {@codes, code};
      endif
    endfor
    menu = {@menu, xterm = "Xterm 256 Value"};
    if (player:ansi_option("truecolor"))
      menu = {@menu, rgb = "24-bit [red]R[normal][green]G[normal][blue]B[normal] Value"};
    endif
    menu = {@menu, "None"};
    sel = $menu_utils:menu(menu, ["hidden_menu" -> input, "input" -> input]);
    if (sel in {0, -1})
      return 0;
    elseif (sel <= length(codes))
      retcode = codes[sel];
      if (retcode[1..2] == "b:")
        retcode = (foreground_color ? foreground_color | "white") + "|" + retcode;
      endif
      if (raw)
        return $ansi_utils:hr_to_code(retcode);
      else
        return retcode;
      endif
    elseif (menu[sel] == "none")
      return "";
    elseif (menu[sel] == xterm)
      player:tell("Please input an Xterm 256 value in between 0-255.");
      val = $command_utils:read();
      xtermint = $code_utils:toint(val);
      if (xtermint == E_TYPE || xtermint > 255 || xtermint < 0)
        return player:tell("Invalid input. You must enter a number that is in between 0 and 255.");
      endif
      if (raw)
        return tostr("[:", xtermint, "]");
      else
        return tostr(":", xtermint);
      endif
    elseif (menu[sel] == rgb)
      red = green = blue = 0;
      invalidmsg = "Invalid color value.";
      while (!red || !green || !blue)
        if (!red)
          player:tell("Enter a [red]red[normal] value between 0 and 255:");
          red = toint($command_utils:read());
          if (red < 0 || red > 255)
            player:tell(invalidmsg);
            red = 0;
            continue;
          endif
        endif
        if (!green)
          player:tell("Enter a [green]green[normal] value between 0 and 255:");
          green = toint($command_utils:read());
          if (green < 0 || green > 255)
            player:tell(invalidmsg);
            green = 0;
            continue;
          endif
        endif
        if (!blue)
          player:tell("Enter a [blue]blue[normal] value between 0 and 255:");
          blue = toint($command_utils:read());
          if (blue < 0 || blue > 255)
            player:tell(invalidmsg);
            blue = 0;
            continue;
          endif
        endif
      endwhile
      if (raw)
        return tostr("[", red, ":", green, ":", blue, "]");
      else
        return tostr(red, ":", green, ":", blue);
      endif
    endif
  endverb

  verb "hr_to_code" (this none this) owner: #36 flags: "rxd"
    "$ansi_utils:hr_to_code(colorstr) - Converts a human readable color sequence to a properly formatted escape code.";
    {colorstr} = args;
    if (colorstr in {0, ""})
      return E_INVARG;
    endif
    if (colorstr[1] == "[")
      colorstr = colorstr[2..$ - 1];
    endif
    if (index(colorstr, "|"))
      ret = "";
      for x in ($string_utils:explode(colorstr, "|"))
        ret = ret + "[" + x + "]";
      endfor
    else
      ret = "[" + colorstr + "]";
    endif
    return ret;
    "Last modified 11/01/18 1:38 a.m. by Sinistral (#2)";
  endverb

endobject
