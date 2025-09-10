object #109
  name: "MCP Package Registry"
  parent: #1
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"MCP Package Registry"};

  property "package_names" (owner: #98, flags: "r") = {"mcp-negotiate", "mcp-cord", "dns-org-mud-moo-simpleedit", "dns-com-vmoo-client", "dns-com-awns-status"};

  property "packages" (owner: #98, flags: "r") = {#105, #106, #113, #121, #123};

  verb "add_package" (this none this) owner: #98 flags: "rxd"
    {name, package} = args;
    if (caller == this || $perm_utils:controls(caller_perms(), this))
      if (name in this.package_names)
        raise(E_INVARG, "Another package with that name already exists");
      elseif (package in this.packages)
        raise(E_INVARG, "That package already is registered under a different name.");
      else
        this.package_names = {@this.package_names, name};
        this.packages = {@this.packages, package};
      endif
    endif
  endverb

  verb "remove_package" (this none this) owner: #98 flags: "rxd"
    {name} = args;
    if (caller == this || $perm_utils:controls(caller_perms(), this))
      if (idx = name in this.package_names)
        this.package_names = listdelete(this.package_names, idx);
        this.packages = listdelete(this.packages, idx);
      else
        raise(E_INVARG, "Not a defined package");
      endif
    endif
  endverb

  verb "match_package" (this none this) owner: #98 flags: "rxd"
    {name} = args;
    if (idx = name in this.package_names)
      return this.packages[idx];
    else
      return $failed_match;
    endif
  endverb

  verb "package_name" (this none this) owner: #98 flags: "rxd"
    {package} = args;
    if (idx = package in this.packages)
      return this.package_names[idx];
    else
      return "";
    endif
  endverb

  verb "packages" (this none this) owner: #98 flags: "rxd"
    return $list_utils:make_alist({this.package_names, this.packages});
  endverb

  verb "init_for_module init_for_core" (this none this) owner: #98 flags: "rxd"
    if (caller_perms().wizard)
      for name in (this.package_names)
        this:remove_package(name);
      endfor
      for x in (children($mcp.package))
        `this:add_package(x.name, x) ! E_INVARG';
      endfor
    endif
  endverb

  verb "@add-package @remove-package" (any at this) owner: #2 flags: "rd"
    if (!$perm_utils:controls(player, this))
      player:tell("You don't have permission to add or remove MCP 2.1 packages.");
    elseif ($command_utils:object_match_failed(dobj, dobjstr))
    elseif (!$object_utils:isa(dobj, $mcp.package))
      player:tell(dobj.name, " is not a valid MCP 2.1 package (descendant of ", $mcp.package, ").");
    elseif (!$perm_utils:controls(player, dobj))
      player:tell("You don't control ", dobj.name, " in order to add or remove it.");
    else
      name = dobj.name;
      package = dobj;
      try
        if (verb == "@add-package")
          this:add_package(name, package);
          player:tell("Added ", package.name, ".");
        else
          this:remove_package(name);
          player:tell("Removed ", package.name, ".");
        endif
      except v (ANY)
        {code, message, value, tb} = v;
        player:tell(code, ": ", message);
      endtry
    endif
  endverb

endobject
