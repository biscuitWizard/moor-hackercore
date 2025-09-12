object #10
  name: "Login Commands"
  parent: #1
  location: #-1
  owner: #2
  readable: true
  override "aliases" = {"Login Commands"};

  override "description" = "This provides everything needed by #0:do_login_command.  See `help $login' on $core_help for details.";

  property "welcome_message" (owner: #2, flags: "rc") = {"Welcome to the HackerCore database.", "", "Type 'connect wizard' to log in.", "", "You will probably want to change this text and the output of the `help' command, which are stored in $login.welcome_message and $login.help_message, respectively."};

  property "newt_registration_string" (owner: #2, flags: "rc") = "Your character is temporarily hosed.";

  property "registration_string" (owner: #2, flags: "rc") = "Character creation is disabled.";

  property "registration_address" (owner: #2, flags: "rc") = "";

  property "create_enabled" (owner: #2, flags: "rc") = 1;

  property "bogus_command" (owner: #2, flags: "r") = "?";

  property "blank_command" (owner: #2, flags: "r") = "welcome";

  property "graylist" (owner: #2, flags: "") = {{}, {}};

  property "blacklist" (owner: #2, flags: "") = {{}, {}};

  property "redlist" (owner: #2, flags: "") = {{}, {}};

  property "who_masks_wizards" (owner: #2, flags: "") = 0;

  property "max_player_name" (owner: #2, flags: "rc") = 40;

  property "spooflist" (owner: #2, flags: "") = {{}, {}};

  property "ignored" (owner: #2, flags: "rc") = {};

  property "max_connections" (owner: #36, flags: "rc") = 99999;

  property "connection_limit_msg" (owner: #36, flags: "r") = "*** The MOO is too busy! The current lag is %l; there are %n connected.  WAIT FIVE MINUTES BEFORE TRYING AGAIN.";

  property "lag_samples" (owner: #2, flags: "rc") = {0, 0, 0, 0, 0};

  property "request_enabled" (owner: #2, flags: "rc") = 0;

  property "help_message" (owner: #2, flags: "rc") = {"Sorry, but there's no help here yet.  Type `?' for a list of commands."};

  property "last_lag_sample" (owner: #2, flags: "rc") = 0;

  property "lag_sample_interval" (owner: #2, flags: "rc") = 15;

  property "lag_cutoff" (owner: #2, flags: "rc") = 5;

  property "lag_exemptions" (owner: #2, flags: "rc") = {};

  property "newted" (owner: #2, flags: "") = {};

  property "current_numcommands" (owner: #2, flags: "rc") = [#-2 -> 2];

  property "max_numcommands" (owner: #2, flags: "rc") = 20;

  property "temporary_newts" (owner: #2, flags: "c") = {};

  property "downtimes" (owner: #2, flags: "rc") = {{1756637231, 0}, {1755930061, 0}};

  property "print_lag" (owner: #2, flags: "rc") = 0;

  property "current_lag" (owner: #2, flags: "r") = 0;

  property "temporary_blacklist" (owner: #2, flags: "") = {{}, {}};

  property "temporary_redlist" (owner: #2, flags: "") = {{}, {}};

  property "temporary_spooflist" (owner: #2, flags: "") = {{}, {}};

  property "temporary_graylist" (owner: #2, flags: "") = {{}, {}};

  property "no_connect_message" (owner: #2, flags: "rc") = 0;

  property "argon2" (owner: #2, flags: "rc") = ["iterations" -> 3, "memory" -> 4096, "threads" -> 1];

  property "name_lookup_players" (owner: #2, flags: "rc") = {};

  verb "?" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
    else
      clist = {};
      for j in ({this, @$object_utils:ancestors(this)})
        for i in [1..length(verbs(j))]
          if (verb_args(j, i) == {"any", "none", "any"} && index((info = verb_info(j, i))[2], "x"))
            vname = $string_utils:explode(info[3])[1];
            star = index(vname + "*", "*");
            clist = {@clist, $string_utils:uppercase(vname[1..star - 1]) + strsub(vname[star..$], "*", "")};
          endif
        endfor
      endfor
      notify(player, "I don't understand that.  Valid commands at this point are");
      notify(player, "   " + $string_utils:english_list(setremove(clist, "?"), "", " or "));
      return 0;
    endif
  endverb

  verb "wel*come @wel*come" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
    else
      msg = this.welcome_message;
      version = server_version();
      for line in (typeof(msg) == LIST ? msg | {msg})
        if (typeof(line) == STR)
          notify(player, strsub(line, "%v", version));
        endif
      endfor
      this:check_for_shutdown();
      this:maybe_print_lag();
      return 0;
    endif
  endverb

  verb "w*ho @w*ho" (any none any) owner: #2 flags: "rxd"
    masked = $login.who_masks_wizards ? $wiz_utils:connected_wizards() | {};
    if (caller != #0 && caller != this)
      return E_PERM;
    elseif (!args)
      plyrs = connected_players();
      if (length(plyrs) > 100)
        this:notify(tostr("You have requested a listing of ", length(plyrs), " players.  Please restrict the number of players in any single request to a smaller number.  The lag thanks you."));
        return 0;
      else
        $ansi_utils:show_who_listing($set_utils:difference(plyrs, masked)) || this:notify("No one logged in.");
      endif
    else
      plyrs = listdelete($command_utils:player_match_result($string_utils:match_player(args), args), 1);
      if (length(plyrs) > 100)
        this:notify(tostr("You have requested a listing of ", length(plyrs), " players.  Please restrict the number of players in any single request to a smaller number.  The lag thanks you."));
        return 0;
      endif
      $ansi_utils:show_who_listing(plyrs, $set_utils:intersection(plyrs, masked));
    endif
    return 0;
  endverb

  verb "cr*eate @cr*eate" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
      "... caller isn't :do_login_command()...";
    elseif (!this:player_creation_enabled(player))
      notify(player, this:registration_string());
      "... we've disabled player creation ...";
    elseif (length(args) != 2)
      notify(player, tostr("Usage:  ", verb, " <new-player-name> <new-password>"));
    elseif ($player_db.frozen)
      notify(player, "Sorry, can't create any new players right now.  Try again in a few minutes.");
    elseif (!(name = args[1]) || name == "<>")
      notify(player, "You can't have a blank name!");
      if (name)
        notify(player, "Also, don't use angle brackets (<>).");
      endif
    elseif (name[1] == "<" && name[$] == ">")
      notify(player, "Try that again but without the angle brackets, e.g.,");
      notify(player, tostr(" ", verb, " ", name[2..$ - 1], " ", strsub(strsub(args[2], "<", ""), ">", "")));
      notify(player, "This goes for other commands as well.");
    elseif (index(name, " "))
      notify(player, "Sorry, no spaces are allowed in player names.  Use dashes or underscores.");
      "... lots of routines depend on there not being spaces in player names...";
    elseif (!$player_db:available(name) || this:_match_player(name) != $failed_match)
      notify(player, "Sorry, that name is not available.  Please choose another.");
      "... note the :_match_player call is not strictly necessary...";
      "... it is merely there to handle the case that $player_db gets corrupted.";
    elseif (!(password = args[2]))
      notify(player, "You must set a password for your player.");
    else
      new = $quota_utils:bi_create($player_class, $nothing);
      set_player_flag(new, 1);
      new.name = name;
      new.aliases = {name};
      new.programmer = $player_class.programmer;
      new.password = $login:encrypt_password(password);
      new.last_password_time = time();
      new.last_connect_time = $maxint;
      "Last disconnect time is creation time, until they login.";
      new.last_disconnect_time = time();
      "make sure the owership quota isn't clear!";
      $quota_utils:initialize_quota(new);
      $player_db:insert(name, new);
      `move(new, $player_start) ! ANY';
      return new;
    endif
    return 0;
  endverb

  verb "_match_player" (this none this) owner: #2 flags: "rxd"
    {possible_id} = args;
    for dude in (players())
      if (dude.name == possible_id || dude.email_address == possible_id)
        return dude;
      endif
    endfor
    return $nothing;
  endverb

  verb "co*nnect @co*nnect" (any none any) owner: #2 flags: "rxd"
    "$login:connect(player-name [, password])";
    " => 0 (for failed connections)";
    " => objnum (for successful connections)";
    caller == #0 || caller == this || raise(E_PERM);
    "=================================================================";
    "=== Check arguments, print usage notice if necessary";
    try
      {name, ?password = 0} = args;
      name = strsub(name, " ", "_");
    except (E_ARGS)
      notify(player, tostr("Usage:  ", verb, " <existing-player-name> <password>"));
      return 0;
    endtry
    try
      "=================================================================";
      "=== Is our candidate name invalid?";
      if (!valid(candidate = orig_candidate = this:_match_player(name)))
        raise(E_INVARG, tostr("`", name, "' matches no player name."));
      endif
      "=================================================================";
      "=== Is our candidate unable to connect for generic security";
      "=== reasons (ie clear password, non-player object)?";
      if (`is_clear_property(candidate, "password") ! E_PROPNF' || !$object_utils:isa(candidate, $player))
        server_log(tostr("FAILED CONNECT: ", name, " (", candidate, ") on ", connection_name(player), $string_utils:connection_hostname(connection_name(player)) in candidate.all_connect_places ? "" | "******"));
        raise(E_INVARG);
      endif
      "=================================================================";
      "=== Check password";
      if (typeof(cp = candidate.password) == STR)
        "=== Candidate requires a password";
        if (password)
          "=== Candidate requires a password, and one was provided";
          if (strcmp(crypt(password, cp), cp))
            "=== Candidate requires a password, and one was provided, which was wrong";
            server_log(tostr("FAILED CONNECT: ", name, " (", candidate, ") on ", connection_name(player), $string_utils:connection_hostname(connection_name(player)) in candidate.all_connect_places ? "" | "******"));
            raise(E_INVARG, "Invalid password.");
          else
            "=== Candidate requires a password, and one was provided, which was right";
          endif
        else
          "=== Candidate requires a password, and none was provided";
          raise(E_INVARG, "A password is required.");
        endif
      elseif (cp == 0)
        "=== Candidate does not require a password";
      else
        "=== Candidate has a nonstandard password; something's wrong";
        raise(E_INVARG);
      endif
      "=================================================================";
      "=== Check guest connections";
      if ($object_utils:isa(candidate, $guest) && !valid(candidate = candidate:defer()))
        if (candidate == #-2)
          server_log(tostr("GUEST DENIED: ", connection_name(player)));
          notify(player, "Sorry, guest characters are not allowed from your site at the current time.");
        else
          notify(player, "Sorry, all of our guest characters are in use right now.");
        endif
        return 0;
      endif
      "=================================================================";
      "=== Check newts";
      if (candidate in this.newted)
        if (entry = $list_utils:assoc(candidate, this.temporary_newts))
          if ((uptime = this:uptime_since(entry[2])) > entry[3])
            "Temporary newting period is over.  Remove entry.  Oh, send mail, too.";
            this.temporary_newts = setremove(this.temporary_newts, entry);
            this.newted = setremove(this.newted, candidate);
            fork (0)
              player = this.owner;
              $mail_agent:send_message(player, $newt_log, tostr("automatic @unnewt ", candidate.name, " (", candidate, ")"), {"message sent from $login:connect"});
            endfork
          else
            notify(player, "");
            notify(player, this:temp_newt_registration_string(entry[3] - uptime));
            boot_player(player);
            return 0;
          endif
        else
          notify(player, "");
          notify(player, this:newt_registration_string());
          boot_player(player);
          return 0;
        endif
      endif
      "=================================================================";
      "=== Log the player on!";
      if (candidate != orig_candidate)
        notify(player, tostr("Okay,... ", name, " is in use.  Logging you in as `", candidate.name, "'"));
      endif
      return candidate;
    except (E_INVARG)
      notify(player, "Either that player does not exist, or has a different password.");
      return 0;
    endtry
  endverb

  verb "q*uit @q*uit" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
    else
      boot_player(player);
      return 0;
    endif
  endverb

  verb "up*time @up*time" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
    else
      notify(player, tostr("The server has been up for ", $time_utils:english_time(time() - $server["last_restart_time"]), "."));
      return 0;
    endif
  endverb

  verb "v*ersion @v*ersion" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
    else
      notify(player, tostr("The MOO is currently running version ", server_version(), " of the ", $server["name"], " server code."));
      return 0;
    endif
  endverb

  verb "parse_command" (this none this) owner: #2 flags: "rxd"
    ":parse_command(@args) => {verb, args}";
    "Given the args from #0:do_login_command,";
    "  returns the actual $login verb to call and the args to use.";
    "Commands available to not-logged-in users should be located on this object and given the verb_args \"any none any\"";
    if (caller != #0 && caller != this)
      return E_PERM;
    endif
    if (!args)
      return {this.blank_command, @args};
    elseif ((verb = args[1]) && !$string_utils:is_numeric(verb))
      for i in ({this, @$object_utils:ancestors(this)})
        try
          if (verb_args(i, verb) == {"any", "none", "any"} && index(verb_info(i, verb)[2], "x"))
            return args;
          endif
        except (ANY)
          continue i;
        endtry
      endfor
    endif
    return {this.bogus_command, @args};
  endverb

  verb "check_for_shutdown" (this none this) owner: #2 flags: "rxd"
    when = $server["shutdown_time"] - time();
    if (when >= 0)
      line = "***************************************************************************";
      notify(player, "");
      notify(player, "");
      notify(player, line);
      notify(player, line);
      notify(player, "****");
      notify(player, "****  WARNING:  The server will shut down in " + $time_utils:english_time(when - when % 60) + ".");
      for piece in ($generic_editor:fill_string($wiz_utils.shutdown_message, 60))
        notify(player, "****            " + piece);
      endfor
      notify(player, "****");
      notify(player, line);
      notify(player, line);
      notify(player, "");
      notify(player, "");
    endif
  endverb

  verb "notify" (this none this) owner: #2 flags: "rxd"
    caller != $ansi_utils && set_task_perms(caller_perms());
    notify(player, $ansi_utils:delete(args[1]));
  endverb

  verb "tell" (this none this) owner: #36 flags: "rxd"
    "keeps bad things from happening if someone brings this object into a room and talks to it.";
    return 0;
  endverb

  verb "player_creation_enabled" (this none this) owner: #2 flags: "rxd"
    "Accepts a player object.  If player creation is enabled for that player object, then return true.  Otherwise, return false.";
    "Default implementation checks the player's connecting host via $login:blacklisted to decide.";
    if (caller_perms().wizard)
      return this.create_enabled && !this:blacklisted($string_utils:connection_hostname(args[1]));
    else
      return E_PERM;
    endif
  endverb

  verb "newt_registration_string registration_string" (this none this) owner: #2 flags: "rxd"
    return $string_utils:subst(this.(verb), {{"%e", this.registration_address}, {"%%", "%"}});
  endverb

  verb "blacklisted graylisted redlisted spooflisted" (this none this) owner: #2 flags: "rxd"
    ":blacklisted(hostname) => is hostname on the .blacklist";
    ":graylisted(hostname)  => is hostname on the .graylist";
    ":redlisted(hostname)   => is hostname on the .redlist";
    sitelist = this.(this:listname(verb));
    if (!caller_perms().wizard)
      return E_PERM;
    elseif ((hostname = args[1]) in sitelist[1] || hostname in sitelist[2])
      return 1;
    elseif ($site_db:domain_literal(hostname))
      for lit in (sitelist[1])
        if (index(hostname, lit) == 1 && (hostname + ".")[length(lit) + 1] == ".")
          return 1;
        endif
      endfor
    else
      for dom in (sitelist[2])
        if (index(dom, "*"))
          "...we have a wildcard; let :match_string deal with it...";
          if ($string_utils:match_string(hostname, dom))
            return 1;
          endif
        else
          "...tail of hostname ...";
          if ((r = rindex(hostname, dom)) && (("." + hostname)[r] == "." && r - 1 + length(dom) == length(hostname)))
            return 1;
          endif
        endif
      endfor
    endif
    return this:(verb + "_temp")(hostname);
  endverb

  verb "blacklist_add*_temp graylist_add*_temp redlist_add*_temp spooflist_add*_temp" (this none this) owner: #2 flags: "rxd"
    "To add a temporary entry, only call the `temp' version.";
    "blacklist_add_temp(Site, start time, duration)";
    if (!caller_perms().wizard)
      return E_PERM;
    endif
    {where, ?start, ?duration} = args;
    lname = this:listname(verb);
    which = 1 + !$site_db:domain_literal(where);
    if (index(verb, "temp"))
      lname = "temporary_" + lname;
      this.(lname)[which] = setadd(this.(lname)[which], {where, start, duration});
    else
      this.(lname)[which] = setadd(this.(lname)[which], where);
    endif
    return 1;
  endverb

  verb "blacklist_remove*_temp graylist_remove*_temp redlist_remove*_temp spooflist_remove*_temp" (this none this) owner: #2 flags: "rxd"
    "The temp version removes from the temporary property if it exists.";
    if (!caller_perms().wizard)
      return E_PERM;
    endif
    where = args[1];
    lname = this:listname(verb);
    which = 1 + !$site_db:domain_literal(where);
    if (index(verb, "temp"))
      lname = "temporary_" + lname;
      if (entry = $list_utils:assoc(where, this.(lname)[which]))
        this.(lname)[which] = setremove(this.(lname)[which], entry);
        return 1;
      else
        return E_INVARG;
      endif
    elseif (where in this.(lname)[which])
      this.(lname)[which] = setremove(this.(lname)[which], where);
      return 1;
    else
      return E_INVARG;
    endif
  endverb

  verb "listname" (this none this) owner: #2 flags: "rxd"
    return {"???", "blacklist", "graylist", "redlist", "spooflist"}[1 + index("bgrs", (args[1] || "?")[1])];
  endverb

  verb "sample_lag" (this none this) owner: #2 flags: "rxd"
    if (!caller_perms().wizard)
      return E_PERM;
    endif
    lag = time() - this.last_lag_sample - 15;
    this.lag_samples = {lag, @this.lag_samples[1..3]};
    "Now compute the current lag and store it in a property, instead of computing it in :current_lag, which is called a hundred times a second.";
    thislag = max(0, time() - this.last_lag_sample - this.lag_sample_interval);
    if (thislag > 60 * 60)
      "more than an hour, probably the lag sampler stopped";
      this.current_lag = 0;
    else
      samples = this.lag_samples;
      sum = 0;
      cnt = 0;
      for x in (listdelete(samples, 1))
        sum = sum + x;
        cnt = cnt + 1;
      endfor
      this.current_lag = max(thislag, samples[1], samples[2], sum / cnt);
    endif
    fork (15)
      this:sample_lag();
    endfork
    this.last_lag_sample = time();
  endverb

  verb "is_lagging" (this none this) owner: #2 flags: "rxd"
    return this:current_lag() > this.lag_cutoff;
  endverb

  verb "max_connections" (this none this) owner: #2 flags: "rxd"
    max = this.max_connections;
    if (typeof(max) == LIST)
      if (this:is_lagging())
        max = max[1];
      else
        max = max[2];
      endif
    endif
    return max;
  endverb

  verb "request_character" (this none this) owner: #2 flags: "rxd"
    "request_character(player, name, address)";
    "return true if succeeded";
    if (!caller_perms().wizard)
      return E_PERM;
    endif
    {who, name, address} = args;
    connection = $string_utils:connection_hostname(who);
    if (reason = $wiz_utils:check_player_request(name, address, connection))
      prefix = "";
      if (reason[1] == "-")
        reason = reason[2..$];
        prefix = "Please";
      else
        prefix = "Please try again, or, to register another way,";
      endif
      notify(who, reason);
      msg = tostr(prefix, " send mail to ", $login.registration_address, ", with the character name you want.");
      for l in ($generic_editor:fill_string(msg, 70))
        notify(who, l);
      endfor
      return 0;
    endif
    if (lines = $no_one:eval_d("$local.help.(\"multiple-characters\")")[2])
      notify(who, "Remember, in general, only one character per person is allowed.");
      notify(who, tostr("Do you already have a ", $network.moo_name, " character? [enter `yes' or `no']"));
      answer = read(who);
      if (answer == "yes")
        notify(who, "Process terminated *without* creating a character.");
        return 0;
      elseif (answer != "no")
        return notify(who, tostr("Please try again; when you get this question, answer `yes' or `no'. You answered `", answer, "'"));
      endif
      notify(who, "For future reference, do you want to see the full policy (from `help multiple-characters'?");
      notify(who, "[enter `yes' or `no']");
      if (read(who) == "yes")
        for x in (lines)
          for y in ($generic_editor:fill_string(x, 70))
            notify(who, y);
          endfor
        endfor
      endif
    endif
    notify(who, tostr("A character named `", name, "' will be created."));
    notify(who, tostr("A random password will be generated, and e-mailed along with"));
    notify(who, tostr(" an explanatory message to: ", address));
    notify(who, tostr(" [Please double-check your email address and answer `no' if it is incorrect.]"));
    notify(who, "Is this OK? [enter `yes' or `no']");
    if (read(who) != "yes")
      notify(who, "Process terminated *without* creating a character.");
      return 0;
    endif
    if (!$network.active)
      $mail_agent:send_message(this.owner, $registration_db.registrar, "Player request", {"Player request from " + connection, ":", "", "@make-player " + name + " " + address});
      notify(who, tostr("Request for new character ", name, " email address '", address, "' accepted."));
      notify(who, tostr("Please be patient until the registrar gets around to it."));
      notify(who, tostr("If you don't get email within a week, please send regular"));
      notify(who, tostr("  email to: ", $login.registration_address, "."));
    elseif ($player_db.frozen)
      notify(who, "Sorry, can't create any new players right now.  Try again in a few minutes.");
    else
      new = $wiz_utils:make_player(name, address);
      password = new[2];
      new = new[1];
      notify(who, tostr("Character ", name, " (", new, ") created."));
      notify(who, tostr("Mailing password to ", address, "; you should get the mail very soon."));
      notify(who, tostr("If you do not get it, please do NOT request another character."));
      notify(who, tostr("Instead, send regular email to ", $login.registration_address, ","));
      notify(who, tostr("with the name of the character you requested."));
      $mail_agent:send_message(this.owner, $new_player_log, tostr(name, " (", new, ")"), {address, tostr(" Automatically created at request of ", valid(player) ? player.name | "unconnected player", " from ", connection, ".")});
      $wiz_utils:send_new_player_mail(tostr("Someone connected from ", connection, " at ", ctime(), " requested a character on ", $network.moo_name, " for email address ", address, "."), name, address, new, password);
      return 1;
    endif
  endverb

  verb "req*uest @req*uest" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
    endif
    "must be #0:do_login_command";
    if (!this.request_enabled)
      for line in ($generic_editor:fill_string(this:registration_string(), 70))
        notify(player, line);
      endfor
    elseif (length(args) != 3 || args[2] != "for")
      notify(player, tostr("Usage:  ", verb, " <new-player-name> for <email-address>"));
    elseif ($login:request_character(player, args[1], args[3]))
      boot_player(player);
    endif
  endverb

  verb "h*elp @h*elp" (any none any) owner: #2 flags: "rxd"
    if (caller != #0 && caller != this)
      return E_PERM;
    else
      msg = this.help_message;
      for line in (typeof(msg) == LIST ? msg | {msg})
        if (typeof(line) == STR)
          notify(player, line);
        endif
      endfor
      return 0;
    endif
  endverb

  verb "maybe_print_lag" (this none this) owner: #2 flags: "rxd"
    if (caller == this || caller_perms() == player)
      if (this.print_lag)
        lag = this:current_lag();
        if (lag > 1)
          lagstr = tostr("approximately ", lag, " seconds");
        elseif (lag == 1)
          lagstr = "approximately 1 second";
        else
          lagstr = "low";
        endif
        notify(player, tostr("The lag is ", lagstr, "; there ", (l = length(connected_players())) == 1 ? "is " | "are ", l, " connected."));
      endif
    endif
  endverb

  verb "current_lag" (this none this) owner: #2 flags: "rxd"
    return this.current_lag;
  endverb

  verb "maybe_limit_commands" (this none this) owner: #2 flags: "rxd"
    "This limits the number of commands that can be issued from the login prompt to prevent haywire login programs from lagging the MOO.";
    "$login.current_numcommands has the number of commands they have issued at the prompt so far.";
    "$login.max_numcommands has the maximum number of commands they may try before being booted.";
    if (!caller_perms().wizard)
      return E_PERM;
    else
      if (!(player in (keys = mapkeys(this.current_numcommands))))
        this.current_numcommands[player] = 1;
      else
        this.current_numcommands[player] = this.current_numcommands[player] + 1;
      endif
      "...sweep idle connections...";
      for p in (keys)
        if (typeof(`idle_seconds(p) ! ANY') == ERR)
          this.current_numcommands = mapdelete(this.current_numcommands, p);
        endif
      endfor
      if (this.current_numcommands[player] > this.max_numcommands)
        notify(player, "Sorry, too many commands issued without connecting.");
        boot_player(player);
        this.current_numcommands = mapdelete(this.current_numcommands, player);
        return 1;
      else
        return 0;
      endif
    endif
  endverb

  verb "on_server_started" (this none this) owner: #2 flags: "rxd"
    "Called by #0:server_started when the server restarts.";
    if (caller_perms().wizard)
      this.lag_samples = {0, 0, 0, 0, 0};
      this.downtimes = {{time(), this.last_lag_sample}, @this.downtimes[1..min($, 100)]};
      this.name_lookup_players = {};
      this.current_numcommands = [];
    endif
  endverb

  verb "uptime_since" (this none this) owner: #2 flags: "rxd"
    "uptime_since(time): How much time the MOO has been up since `time'";
    since = args[1];
    up = time() - since;
    for x in (this.downtimes)
      if (x[1] < since)
        "downtime predates when we're asking about";
        return up;
      endif
      "since the server was down between x[2] and x[1], don't count it as uptime";
      up = up - (x[1] - max(x[2], since));
    endfor
    return up;
  endverb

  verb "count_bg_players" (this none this) owner: #2 flags: "rxd"
    caller_perms().wizard || $error:raise(E_PERM);
    now = time();
    tasks = queued_tasks();
    sum = 0;
    for t in (tasks)
      delay = t[2] - now;
      interval = delay <= 0 ? 1 | delay * 2;
      "SUM is measured in hundredths of a player for the moment...";
      delay <= 300 && (sum = sum + 2000 / interval);
    endfor
    count = sum / 100;
    return count;
  endverb

  verb "blacklisted_temp graylisted_temp redlisted_temp spooflisted_temp" (this none this) owner: #2 flags: "rxd"
    ":blacklisted_temp(hostname) => is hostname on the .blacklist...";
    ":graylisted_temp(hostname)  => is hostname on the .graylist...";
    ":redlisted_temp(hostname)   => is hostname on the .redlist...";
    ":spooflisted_temp(hostname) => is hostname on the .spooflist...";
    "";
    "... and the time limit hasn't run out.";
    lname = this:listname(verb);
    sitelist = this.("temporary_" + lname);
    if (!caller_perms().wizard)
      return E_PERM;
    elseif (entry = $list_utils:assoc(hostname = args[1], sitelist[1]))
      return this:templist_expired(lname, @entry);
    elseif (entry = $list_utils:assoc(hostname, sitelist[2]))
      return this:templist_expired(lname, @entry);
    elseif ($site_db:domain_literal(hostname))
      for lit in (sitelist[1])
        if (index(hostname, lit[1]) == 1 && (hostname + ".")[length(lit[1]) + 1] == ".")
          return this:templist_expired(lname, @lit);
        endif
      endfor
    else
      for dom in (sitelist[2])
        if (index(dom[1], "*"))
          "...we have a wildcard; let :match_string deal with it...";
          if ($string_utils:match_string(hostname, dom[1]))
            return this:templist_expired(lname, @dom);
          endif
        else
          "...tail of hostname ...";
          if ((r = rindex(hostname, dom[1])) && (("." + hostname)[r] == "." && r - 1 + length(dom[1]) == length(hostname)))
            return this:templist_expired(lname, @dom);
          endif
        endif
      endfor
    endif
    return 0;
  endverb

  verb "templist_expired" (this none this) owner: #2 flags: "rxd"
    "check to see if duration has expired on temporary_<colorlist>. Removes entry if so, returns true if still <colorlisted>";
    ":(listname, hostname, start time, duration)";
    {lname, hname, start, duration} = args;
    if (!caller_perms().wizard)
      return E_PERM;
    endif
    if (this:uptime_since(start) > duration)
      this:(lname + "_remove_temp")(hname);
      return 0;
    else
      return 1;
    endif
  endverb

  verb "temp_newt_registration_string" (this none this) owner: #2 flags: "rxd"
    return "Your character is unavailable for another " + $time_utils:english_time(args[1]) + ".";
  endverb

  verb "do_out_of_band_command doobc" (this none this) owner: #36 flags: "rxd"
    "This is where oob handlers need to be put to handle oob commands issued prior to assigning a connection to a player object.  Right now it simply returns.";
    return;
  endverb

  verb "connection_name_lookup" (this none this) owner: #2 flags: "rxd"
    ":connection_name_lookup(connection)";
    "Perform a threaded DNS lookup on 'connection' and record it to avoid multiple calls.";
    if (caller != #0 && caller != this)
      return E_PERM;
    endif
    {connection} = args;
    if (!(connection in this.name_lookup_players))
      this.name_lookup_players = setadd(this.name_lookup_players, connection);
      try
        result = connection_name_lookup(connection, 1);
        if (typeof(result) == MAP)
          $mail_agent:send_messsage(this, $wiz_utils.gripe_recipients, {"connection_name_lookup error", $wiz_utils.gripe_recipients}, {result["error"], result["message"]});
        endif
      except (E_INVARG, E_QUOTA)
        "Not a critical error, but you may want to do something with it here.";
      endtry
    endif
  endverb

  verb "delete_name_lookup" (this none this) owner: #2 flags: "rxd"
    ":delete_name_lookup(connection)";
    "Remove a connection from the list of connections that have already have name lookups performed on.";
    if (caller != #0 && caller != this)
      return E_PERM;
    endif
    {connection} = args;
    this.name_lookup_players = setremove(this.name_lookup_players, connection);
  endverb

  verb "mssp-request" (any none any) owner: #2 flags: "rxd"
    "Return MSSP variables using the plaintext fallback protocol.";
    if (caller != $sysobj && caller != this)
      return E_PERM;
    elseif (!$telnet.mssp_active)
      return 1;
    else
      tab = $string_utils.tab;
      mssp_data = $telnet:_mssp_data();
      notify(player, "");
      notify(player, "MSSP-REPLY-START");
      for value, key in (mssp_data)
        notify(player, tostr(key, tab, value));
      endfor
      notify(player, "MSSP-REPLY-END");
      return 0;
    endif
  endverb

endobject
