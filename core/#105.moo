object #105
  name: "mcp-negotiate"
  parent: #104
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"mcp-negotiate", "negotiate"};

  override "messages_in" = {{"can", {"package", "min-version", "max-version"}}, {"end", {}}};

  override "messages_out" = {{"can", {"package", "min-version", "max-version"}}, {"end", {}}};

  override "version_range" = {"1.0", "2.0"};

  verb "do_negotiation" (this none this) owner: #98 flags: "rxd"
    connection = caller;
    for keyval in ($mcp.registry:packages())
      {name, package} = keyval;
      this:send_can(connection, name, @package.version_range);
    endfor
    this:send_end(connection);
  endverb

  verb "handle_can" (this none this) owner: #98 flags: "rxd"
    if (caller == this)
      {connection, package, minv, maxv, @rest} = args;
      if (valid(pkg = $mcp.registry:match_package(package)))
        if (version = $mcp:compare_version_range({minv, maxv}, pkg.version_range))
          connection:add_package(pkg, version);
        endif
      endif
    endif
  endverb

  verb "handle_end" (this none this) owner: #98 flags: "rxd"
    if (caller == this)
      {connection, @rest} = args;
      connection:end_negotiation();
    endif
  endverb

endobject
