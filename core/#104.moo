object #104
  name: "Generic MCP Package"
  parent: #103
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"Generic MCP Package"};

  override "messages_out" = {};

  property "version_range" (owner: #98, flags: "r") = {"1.0", "1.0"};

  property "cord_types" (owner: #98, flags: "r") = {};

  verb "set_version_range" (this none this) owner: #98 flags: "rx"
    "This is the standard :set_foo verb.  It allows the property to be set if called by this or called with adequate permissions (this's owner or wizardly).";
    if (caller == this || $perm_utils:controls(caller_perms(), this))
      return this.(verb[5..length(verb)]) = args[1];
    else
      return E_PERM;
    endif
    "version: 1.0 Fox Wed Jul  5 17:58:13 1995 EDT";
  endverb

  verb "dispatch" (this none this) owner: #2 flags: "rxd"
    "Usage:  :dispatch_request(who, authkey, name, arguments)";
    "";
    connection = caller;
    {message, alist} = args;
    if (verbname = this:message_name_to_verbname(message))
      set_task_perms(caller_perms());
      this:(verbname)(connection, @this:parse_receive_args(message, alist));
    endif
  endverb

  verb "match_request" (this none this) owner: #98 flags: "rxd"
    "Usage:  :match_request(request)";
    "";
    request = args[1];
    if ($object_utils:has_verb(this, verbname = "mcp_" + request))
      return verbname;
    else
      return 0;
    endif
    "version: 1.0 Fox Wed Jul  5 17:58:14 1995 EDT";
  endverb

  verb "initialize_connection" (this none this) owner: #98 flags: "rxd"
    "Usage:  :initialize_connection()";
    "";
    {version} = args;
    connection = caller;
    messages = $list_utils:slice(this.messages_in);
    connection:register_handlers(messages);
  endverb

  verb "message_name_to_verbname" (this none this) owner: #98 flags: "rxd"
    {message} = args;
    if ($object_utils:has_callable_verb(this, vname = "handle_" + message))
      return vname;
    else
      return 0;
    endif
  endverb

  verb "finalize_connection" (this none this) owner: #98 flags: "rxd"
    connection = caller;
    return 0;
  endverb

  verb "add_cord_type" (this none this) owner: #98 flags: "rxd"
    "Usage:  :add_cord_type(cord_type)";
    "";
    {cord_type} = args;
    if (caller == this || $perm_utils:controls(caller_perms(), this))
      return this.cord_types = setadd(this.cord_types, cord_type);
    else
      raise(E_PERM);
    endif
  endverb

  verb "remove_cord_type" (this none this) owner: #98 flags: "rxd"
    "Usage:  :remove_cord_type(cord_type)";
    "";
    {cord_type} = args;
    if (caller == this || $perm_utils:controls(caller_perms(), this))
      return this.cord_types = setremove(this.cord_types, cord_type);
    else
      raise(E_PERM);
    endif
  endverb

  verb "send_*" (this none this) owner: #98 flags: "rxd"
    {connection, @args} = args;
    if (caller == this)
      message = verb[6..$];
      `connection:send(message, this:parse_send_args(message, @args)) ! E_VERBNF';
    else
      raise(E_PERM);
    endif
  endverb

endobject
