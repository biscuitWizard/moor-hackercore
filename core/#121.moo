object #121
  name: "dns-com-vmoo-client"
  parent: #104
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"dns-com-vmoo-client"};

  override "messages_in" = {{"screensize", {"cols", "rows"}}, {"info", {"name", "text-version", "internal-version"}}};

  override "messages_out" = {{"disconnect", {"reason"}}};

  property "client_info" (owner: #98, flags: "") = {};

  property "client_stats" (owner: #98, flags: "rc") = {};

  verb "handle_info" (this none this) owner: #98 flags: "rxd"
    if (caller != this)
      return E_PERM;
    endif
    {session, @stats} = args;
    who = session.connection;
    if (i = who in $list_utils:slice(this.client_info, 2))
      this.client_info = listset(this.client_info, {time(), who, stats}, i);
    else
      this.client_info = {@this.client_info, {time(), who, stats}};
    endif
  endverb

  verb "handle_screensize" (this none this) owner: #98 flags: "rxd"
    {session, linelen, @args} = args;
    if (caller != this)
      return E_PERM;
    endif
    (ll = toint(linelen)) > 0 && this:adjust_linelen(who = session.connection, who.linelen > 0 ? ll | -1 * ll);
  endverb

  verb "send_disconnect" (this none this) owner: #98 flags: "rxd"
    {who, @args} = args;
    if ($perm_utils:controls(caller_perms(), who))
      if (valid(session = $mcp:session_for(who)) && session:handles_package(this))
        return pass(session, @args);
      else
        return E_INVIND;
      endif
    endif
  endverb

  verb "get_client_info" (this none this) owner: #98 flags: "rxd"
    if (!$perm_utils:controls(caller_perms(), this))
      return E_PERM;
    else
      info = {};
      for dude in (args)
        if (i = $list_utils:iassoc(dude, this.client_info, 2))
          ticks_left() < 4000 && suspend(0);
          dudeinf = this.client_info[i];
          session = $mcp:session_for(dudeinf[2]);
          if (valid(session) && session:handles_package(this) && `dudeinf[1] >= dude.last_connect_time - 3 ! ANY')
            info = {@info, dudeinf[2..3]};
          endif
        endif
      endfor
      return info;
    endif
  endverb

  verb "adjust_linelen" (this none this) owner: #2 flags: "rxd"
    {who, linelen} = args;
    if (caller != this)
      return E_PERM;
    endif
    who.linelen = linelen;
  endverb

  verb "prune_client_info" (this none this) owner: #98 flags: "rxd"
    {clients, users} = this.client_stats;
    for x in (this.client_info)
      if (is_player(x[2]) == 0)
        if ((ind = x[3][1] in clients) == 0)
          clients = {@clients, x[3][1]};
          users = {@users, 1};
        else
          users[ind] = users[ind] + 1;
        endif
        this.client_info = setremove(this.client_info, x);
      endif
    endfor
    this.client_stats = {clients, users};
  endverb

endobject
