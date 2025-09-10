object #107
  name: "MCP 2.1 Parser"
  parent: #1
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"MCP 2.1 Parser"};

  property "next_datakey" (owner: #98, flags: "r") = 10202;

  property "unquoted_string" (owner: #2, flags: "r") = "^[]a-zA-Z0-9-%~`!@#$^&()=+{}[|';?/><.,]+$";

  verb "parse_mcp_alist" (this none this) owner: #98 flags: "rxd"
    "take args and return a list in the format:";
    "{true if contains multiline, { { keyword-name, data, multiline }, ... }";
    alist = {};
    if (length(alist) % 2)
      raise(E_ARGS);
    endif
    contains_multiline = 0;
    while (args)
      {keyword, value, @args} = args;
      if (keyword[$] != ":")
        raise(E_INVARG, "invalid keyword: " + keyword);
      else
        if (keyword[$ - 1] == "*")
          contains_multiline = 1;
          value = {};
          keyword = keyword[1..$ - 2];
        else
          keyword = keyword[1..$ - 1];
        endif
        alist = {@alist, {keyword, value}};
      endif
    endwhile
    return {contains_multiline, alist};
  endverb

  verb "parse_mcp" (this none this) owner: #98 flags: "rxd"
    "parse_mcp(@args) =>";
    "relies on argstr being a version of @args unwordified";
    "{request-name, contains-multiline, authentication-key, data-tag, { { keyword-name, data }, ... } }";
    if (length(args) < 1)
      raise(E_INVARG, "not enough arguments");
    endif
    request_name = args[1][4..$];
    if (!request_name)
      raise(E_INVARG, "no request name");
    endif
    if (request_name == "*")
      return this:parse_mcp_continuation(@args[2..$]);
    endif
    "... if there is an authentication key, the length of args will be even ...";
    if (length(args) % 2)
      authentication_key = E_NONE;
      message_args = args[2..$];
    else
      authentication_key = args[2];
      message_args = args[3..$];
    endif
    {contains_multiline, alist} = this:parse_mcp_alist(@message_args);
    if (contains_multiline)
      if (tag = $list_utils:iassoc("_data-tag", alist))
        "mulitline with a datatag, OK";
        data_tag = alist[tag][2];
        alist = listdelete(alist, tag);
      else
        raise(E_INVARG, "multiline fields with no data tag");
      endif
    else
      data_tag = E_NONE;
    endif
    if (typeof(alist) == LIST)
      return {request_name, contains_multiline, authentication_key, data_tag, alist};
    else
      return alist;
    endif
  endverb

  verb "parse_mcp_continuation" (this none this) owner: #98 flags: "rxd"
    {data_tag, keyword, @rest} = args;
    value = argstr[index(argstr, keyword) + length(keyword) + 1..$];
    keyword = keyword[1..$ - 1];
    return {"*", data_tag, keyword, value};
  endverb

  verb "parse" (this none this) owner: #98 flags: "rxd"
    "parse(@args) => parsed MCP message ready for dispatch or 0";
    "                if there was nothing to dispatch for this message";
    "                (as in multiline continuations, dispatch";
    "                for those occurs at the END";
    "returns {message, authkey, alist} or 0";
    "argstr must equal the unmodified line from the client";
    {argstr, @words} = args;
    session = caller;
    message = this:parse_mcp(@words);
    if (message[1] == "*")
      {n, data_tag, keyword, value} = message;
      session:multiline_add_value(data_tag, keyword, value);
    elseif (message[1] == ":" || message[1] == "END")
      {request, dummy, data_tag, dummy, dummy} = message;
      return session:multiline_finish(player, data_tag);
    else
      {request, contains_multiline, authkey, data_tag, alist} = message;
      if (contains_multiline)
        session:multiline_begin(request, authkey, data_tag, alist);
      else
        return {request, authkey, alist};
      endif
    endif
    return 0;
  endverb

  verb "unparse" (this none this) owner: #98 flags: "rxd"
    {request, authkey, alist} = args;
    keyvals = "";
    need_data_tag = 0;
    multilines = {};
    for keyval in (alist)
      {keyword, value, ?maybe_ignore} = keyval;
      if (typeof(value) == STR)
        if (!match(value, this.unquoted_string))
          value = toliteral(value);
        endif
      elseif (typeof(value) == LIST)
        need_data_tag = 1;
        multilines = {@multilines, {keyword, value}};
        keyword = keyword + "*";
        value = "\"\"";
      else
        value = toliteral(value);
      endif
      keyvals = keyvals + " " + keyword + ": " + value;
    endfor
    if (need_data_tag)
      data_tag = this:next_datakey();
      keyvals = keyvals + " _data-tag: " + data_tag;
    endif
    message = "#$#" + request;
    if (authkey)
      message = message + " " + authkey;
    endif
    message = {message + keyvals};
    if (need_data_tag)
      prefix = "#$#* " + data_tag + " ";
      for field in (multilines)
        {keyword, value} = field;
        for line in (value)
          message = {@message, tostr(prefix, keyword, ": ", typeof(line) == LIST ? toliteral(line) | line)};
        endfor
      endfor
      message = {@message, "#$#: " + data_tag};
    endif
    return message;
  endverb

  verb "next_datakey" (this none this) owner: #98 flags: "rxd"
    datakey = tostr(random(), this.next_datakey);
    this.next_datakey = this.next_datakey + 1;
    return datakey;
  endverb

endobject
