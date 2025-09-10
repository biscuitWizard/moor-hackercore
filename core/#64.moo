object #64
  name: "Telnet Handler"
  parent: #1
  location: #-1
  owner: #2
  readable: true
  override "key" = 0;

  override "aliases" = {"Telnet Handler", "Handler"};

  property "commands" (owner: #2, flags: "rc") = ["DO" -> 253, "DONT" -> 254, "GMCP" -> 201, "IAC" -> 255, "MCCP" -> 86, "MSDP" -> 69, "MSSP" -> 70, "MSSP_VAL" -> 2, "MSSP_VAR" -> 1, "SB" -> 250, "SE" -> 240, "TTYPE" -> 24, "WILL" -> 251, "WONT" -> 252];

  property "handled_commands" (owner: #2, flags: "rc") = [70 -> "do_mssp"];

  property "mssp_data" (owner: #2, flags: "rc") = ["codebase" -> 1, "contact" -> 1, "family" -> "MOO", "hostname" -> 1, "name" -> 1, "players" -> 1, "uptime" -> 1];

  property "mssp_active" (owner: #2, flags: "rc") = 1;

  verb "do_out_of_band_command doobc" (this none this) owner: #2 flags: "rxd"
    "Process out of band data for telnet command sequences.";
    "As of right now, only 'IAC DO' command sequences are processed. You can, of course, extend this verb to support other telnet commands as needed.";
    if (caller != $sysobj)
      raise(E_PERM);
    endif
    cmd = decode_binary(args[1], 1);
    "Don't process commands that don't begin with 'IAC DO'.";
    if (length(cmd) < 3 || cmd[1..2] != {this.commands["IAC"], this.commands["DO"]})
      return 0;
    endif
    ".handled_commands maps telnet options to MOO verbs. e.g. '70' is the MSSP option, which maps to the 'do_mssp' verb.";
    if (cmd[3] in mapkeys(this.handled_commands))
      return this:(this.handled_commands[cmd[3]])(cmd[3..$]);
    else
      return 0;
    endif
  endverb

  verb "do_mssp" (this none this) owner: #2 flags: "rxd"
    ":do_mssp(LIST <telnet commands>) => INT <successfully handled>";
    "Implements the MUD Server Status Protocol via telnet.";
    "'player' is sent binary data and this verb returns 1 if successful, 0 if not.";
    "See 'HELP MSSP' for details on how to populate MSSP variables.";
    if (caller != this)
      raise(E_PERM);
    endif
    {telnet_commands} = args;
    MSSP_VAR = this.commands["MSSP_VAR"];
    MSSP_VAL = this.commands["MSSP_VAL"];
    mssp = {this.commands["IAC"], this.commands["SB"], this.commands["MSSP"]};
    mssp_data = this:_mssp_data();
    for value, key in (mssp_data)
      mssp = {@mssp, MSSP_VAR, key, MSSP_VAL, value};
    endfor
    mssp = {@mssp, this.commands["IAC"], this.commands["SE"]};
    return this:send(player, mssp);
  endverb

  verb "mssp_name" (this none this) owner: #2 flags: "rxd"
    return $network.moo_name;
  endverb

  verb "mssp_codebase" (this none this) owner: #2 flags: "rxd"
    return "LambdaMOO-ToastStunt " + server_version();
  endverb

  verb "mssp_players" (this none this) owner: #2 flags: "rxd"
    return length(connected_players());
  endverb

  verb "_mssp_data" (this none this) owner: #2 flags: "rxd"
    ":_mssp_data() => MAP <mssp data>";
    "Return a map of the MSSP data that we actually have available.";
    mssp = [];
    for value, key in (this.mssp_data)
      if (value == $nothing)
        continue;
      elseif (value == 1)
        mssp_verb = tostr("mssp_", key);
        if ($object_utils:has_callable_verb(this, mssp_verb))
          value = `this:(mssp_verb)() ! ANY => $nothing';
          if (value == $nothing)
            continue;
          endif
        endif
      endif
      mssp[key] = toliteral(value);
    endfor
    return mssp;
  endverb

  verb "mssp_uptime" (this none this) owner: #2 flags: "rxd"
    return $server["last_restart_time"];
  endverb

  verb "mssp_contact" (this none this) owner: #2 flags: "rxd"
    if ($network.postmaster != "postmastername@yourhost")
      return $network.postmaster;
    else
      return $nothing;
    endif
  endverb

  verb "mssp_hostname" (this none this) owner: #2 flags: "rxd"
    if ($network.site != "yoursite")
      return $network.site;
    else
      return $nothing;
    endif
  endverb

endobject
