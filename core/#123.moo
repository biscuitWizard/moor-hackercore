object #123
  name: "dns-com-awns-status"
  parent: #104
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"dns-com-awns-status"};

  override "messages_out" = {{"", {"text"}}};

  override "version_range" = {"1.0", "1.0"};

  verb "status" (this none this) owner: #98 flags: "rxd"
    {session, text} = args;
    if (caller == this || $perm_utils:controls(caller_perms(), session.connection))
      return this:send_(session, text);
    endif
  endverb

endobject
