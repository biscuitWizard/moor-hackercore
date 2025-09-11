object #110
  name: "MCP 2.1"
  parent: #1
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"MCP 2.1", "mcp"};

  property "parser" (owner: #98, flags: "rc") = #107;

  property "session" (owner: #98, flags: "r") = #108;

  property "version" (owner: #98, flags: "rc") = {2, 1};

  property "package" (owner: #2, flags: "r") = #104;

  property "negotiate" (owner: #2, flags: "r") = #105;

  property "registry" (owner: #98, flags: "rc") = #109;

  property "cord" (owner: #98, flags: "r") = #106;

  property "simpleedit" (owner: #98, flags: "rc") = #113;

  property "client" (owner: #98, flags: "rc") = #121;

  property "status" (owner: #98, flags: "rc") = #123;

  property "dispatch" (owner: #98, flags: "rc") = #103;

  verb "create_session" (this none this) owner: #98 flags: "rxd"
    {connection} = args;
    if (caller != this)
      raise(E_PERM);
    elseif (typeof(session = this.session:new()) != ANON)
      raise(session);
    endif
    session:set_connection(connection);
    session:initialize_connection();
    return session;
  endverb

  verb "destroy_session" (this none this) owner: #98 flags: "rxd"
    {session} = args;
    if (!(caller in {this, session}))
      raise(E_PERM);
    elseif (!$object_utils:isa(session, this.session))
      raise(E_INVARG);
    elseif (session == this.session)
      raise(E_INVARG);
    else
      $recycler:_recycle(session);
    endif
  endverb

  verb "initialize_connection" (this none this) owner: #2 flags: "rxd"
    {who} = args;
    if (caller != this)
      raise(E_PERM);
    endif
    return this:create_session(who);
  endverb

  verb "finalize_connection" (this none this) owner: #2 flags: "rxd"
    {con} = args;
    if (caller == con)
      this:destroy_session(con);
    endif
  endverb

  verb "parse_version" (this none this) owner: #2 flags: "rxd"
    "string version number -> {major, minor}";
    {version} = args;
    if (m = match(version, "%([0-9]+%)%.%([0-9]+%)"))
      return {toint(substitute("%1", m)), toint(substitute("%2", m))};
    endif
  endverb

  verb "compare_version_range" (this none this) owner: #2 flags: "rxd"
    {client, server} = args;
    {min1, max1} = client;
    {min2, max2} = server;
    min1 = typeof(min1) == STR ? this:parse_version(min1) | min1;
    min2 = typeof(min2) == STR ? this:parse_version(min2) | min2;
    max1 = typeof(max1) == STR ? this:parse_version(max1) | max1;
    max2 = typeof(max2) == STR ? this:parse_version(max2) | max2;
    if (!(min1 && min2 && max1 && max2))
      return;
    else
      if (this:compare_version(max1, min2) <= 0 && this:compare_version(max2, min1) <= 0)
        if (this:compare_version(max1, max2) < 0)
          return max2;
        else
          return max1;
        endif
      endif
    endif
    return 0;
  endverb

  verb "compare_version" (this none this) owner: #2 flags: "rxd"
    "-1 if v1 > v2, 0 if v1 = v2, 1 if v1 < v2";
    {v1, v2} = args;
    if (v1 == v2)
      return 0;
    else
      {major1, minor1} = v1;
      {major2, minor2} = v2;
      if (major1 == major2)
        if (minor1 > minor2)
          return -1;
        else
          return 1;
        endif
      elseif (major1 > major2)
        return -1;
      else
        return 1;
      endif
    endif
  endverb

  verb "unparse_version" (this none this) owner: #2 flags: "rxd"
    {major, minor} = args;
    return tostr(major, ".", minor);
  endverb

  verb "session_for" (this none this) owner: #2 flags: "rxd"
    {who} = args;
    return `who.out_of_band_session ! E_PROPNF => $failed_match';
  endverb

  verb "user_created user_connected user_reconnected" (this none this) owner: #2 flags: "rxd"
    {who} = args;
    if (listeners(caller))
      if ($recycler:valid(who.out_of_band_session))
        `who.out_of_band_session:finish() ! ANY';
      endif
      who.out_of_band_session = this:initialize_connection(who);
    endif
  endverb

  verb "user_disconnected user_client_disconnected" (this none this) owner: #2 flags: "rxd"
    {who} = args;
    if (listeners(caller))
      if ($recycler:valid(who.out_of_band_session))
        `who.out_of_band_session:finish() ! ANY';
        who.out_of_band_session = $nothing;
      endif
    else
      raise(E_PERM);
    endif
  endverb

  verb "do_out_of_band_command" (this none this) owner: #2 flags: "rxd"
    if (listeners(caller))
      if ($recycler:valid(session = player.out_of_band_session))
        set_task_perms(player);
        return session:do_out_of_band_command(@args);
      endif
    endif
  endverb

  verb "package_name match_package packges" (this none this) owner: #98 flags: "rxd"
    return this.registry:(verb)(@args);
  endverb

  verb "handles_package wait_for_package" (this none this) owner: #98 flags: "rxd"
    {who, @rest} = args;
    if (valid(session = this:session_for(who)))
      return session:(verb)(@rest);
    endif
  endverb

endobject
