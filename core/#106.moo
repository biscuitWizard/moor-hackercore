object #106
  name: "mcp-cord"
  parent: #104
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"mcp-cord", "cord"};

  override "messages_in" = {{"open", {"_id", "_type"}}, {"", {"_id", "_message"}}, {"closed", {"_id"}}};

  override "messages_out" = {{"open", {"_id", "_type"}}, {"", {"_id", "_message"}}, {"closed", {"_id"}}};

  override "cord_types" = {};

  property "next_id" (owner: #98, flags: "r") = 1;

  property "cords" (owner: #98, flags: "r") = {};

  property "registry_ids" (owner: #98, flags: "rc") = {};

  verb "next_id" (this none this) owner: #98 flags: "rxd"
    if (caller == this)
      return tostr("I", this.next_id = this.next_id + 1);
    else
      raise(E_PERM);
    endif
  endverb

  verb "cord_send" (this none this) owner: #2 flags: "rxd"
    {message, alist} = args;
    cord = caller;
    session = cord.session;
    if (cord in $mcp.cord.registry)
      return this:send_(session, tostr(cord.id), message, @alist);
    else
      raise(E_PERM);
    endif
  endverb

  verb "cord_closed" (this none this) owner: #2 flags: "rxd"
    cord = caller;
    session = cord.session;
    this:send_closed(session, tostr(cord.id));
  endverb

  verb "handle_" (this none this) owner: #98 flags: "rxd"
    {session, id, message, @assocs} = args;
    if (caller == this)
      $mcp.cord:mcp_receive(id, message, assocs);
    endif
  endverb

  verb "handle_closed" (this none this) owner: #98 flags: "rxd"
    {session, id, @rest} = args;
    if (caller == this)
      $mcp.cord:mcp_closed(id);
    endif
  endverb

  verb "find_type" (this none this) owner: #98 flags: "rxd"
    {name} = args;
    for i in ($object_utils:leaves($mcp.cord.type_root))
      if (name == $mcp.registry:package_name(i.parent_package) + "-" + i.cord_name)
        return i;
      endif
    endfor
    return $failed_Match;
  endverb

  verb "send_open" (this none this) owner: #98 flags: "rxd"
    if (caller == $mcp.cord)
      return pass(@args);
    endif
  endverb

  verb "finalize_connection" (this none this) owner: #2 flags: "rxd"
    session = caller;
    len = length($mcp.cord.registry_ids);
    for i in [0..len - 1]
      idx = len - i;
      cord = $mcp.cord.registry[idx];
      if (cord.session == session)
        $recycler:_recycle(cord);
      endif
    endfor
  endverb

  verb "type_name" (this none this) owner: #98 flags: "rxd"
    {cord_type} = args;
    parent = $mcp:package_name(cord_type.parent_package);
    if (suffix = cord_type.cord_name)
      return parent + "-" + suffix;
    else
      return parent;
    endif
  endverb

endobject
