object #19
  name: "Generic Property-Based Datastore"
  parent: #17
  location: #-1
  owner: #36
  readable: true

  property "key_prefix" (owner: #36, flags: "r") = "_";

  property "columns" (owner: #36, flags: "r") = {};

  verb "retrieve" (this none this) owner: #36 flags: "rxd"
    ":retrieve(OBJ/STR/FLOAT/INT key) => ANY value";
    "  retrieves data located at key";
    {key} = args;
    key = this:format_key(key);
    try
      return this.(key);
    except e (E_PROPNF, E_RANGE)
      raise(E_NONE, tostr("Unable to find data at key '", key, "'."));
    endtry
  endverb

  verb "keys" (this none this) owner: #36 flags: "rxd"
    ":keys() => LIST of all keys present on datastore";
    keys = {};
    for prop in (properties(this))
      if (this.key_prefix in prop != 1)
        continue;
      endif
      keys = {@keys, prop};
    endfor
    return keys;
  endverb

  verb "delete" (this none this) owner: #36 flags: "rxd"
  endverb

  verb "save" (this none this) owner: #36 flags: "rxd"
    ":save(OBJ/STR/INT/FLOAT key, ANY value) => NONE";
    "  saves a key as a property";
    {key, value} = args;
    formatted_key = this:format_key(key);
    try
      this.(formatted_key) = value;
    except e (E_PROPNF)
      add_property(this, formatted_key, value, {this.owner, "r"});
    endtry
  endverb

  verb "has_key" (this none this) owner: #36 flags: "rxd"
    ":has(OBJ/STR/FLOAT/INT key) => BOOL if key exits";
    "  retrieves data located at key";
    {key} = args;
    key = this:format_key(key);
    return $ou:has_property(this, key);
  endverb

  verb "format_key" (this none this) owner: #36 flags: "rxd"
    ":format_key(OBJ/STR/INT/FLOAT key) => STR key";
    "  converts a key into proper format";
    {key} = args;
    if (!(typeof(key) in {OBJ, STR, INT, FLOAT}))
      raise(E_INVARG, "Invalid key type provided.");
    endif
    return tostr(this.key_prefix, $su:lowercase(pcre_replace(tostr(key), "s/\\s/_/g")));
  endverb

endobject
