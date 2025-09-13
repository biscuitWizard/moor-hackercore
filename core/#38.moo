
object #38
  name: "Generic Property-Based Columnar Datastore"
  parent: #37
  location: #84
  owner: #36
  readable: true

  property "key_prefix" (owner: #36, flags: "r") = "_";

  verb "save" (this none this) owner: #36 flags: "rxd"
    ":save(OBJ/STR/INT/FLOAT key, LIST values) => NONE";
    "  Adds or updates a value in the datastore";
    {key, @data} = args;
    if (length(data) == 1)
      "this handles cases when values is a straight list";
      "it's also acceptable for data to come as a list of args.";
      data = data[1];
    endif
    this:validate_row(data);
    key = this:format_key(key);
    try
      this.(key) = data;
    except e (E_PROPNF)
      add_property(this, key, data, {this.owner, "r"});
    endtry
  endverb

  verb "save_at" (this none this) owner: #36 flags: "rxd"
    ":save_at(STR key, INT/STR column_index, ANY value) => ROW LIST values";
    "  Saves a value at a specific cell.";
    "  column index is 1-based, not 0-based.";
    {raw_key, column_index, value} = args;
    column_index = this:resolve_column_index(column_index);
    "check for typing";
    if (are_columns_typed || $lu:is_assoc(this.columns))
      "check that the value is correct for column type";
      valid_types = this.columns[column_index][2];
      valid_types = typeof(valid_types) == LIST ? valid_types | {valid_types};
      if (!(typeof(value) in valid_types))
        raise(E_INVARG, tostr("Unable to save to column ", column_index, ", expected ", $su:english_list(valid_types), " but got ", typeof(value), "."));
      endif
    endif
    key = this:format_key(raw_key);
    this.(key)[column_index] = value;
    try
      return this.(key);
    except e (E_PROPNF)
      raise(E_NONE, tostr("No record found for key ", raw_key, "."));
    endtry 
  endverb

  verb "retrieve" (this none this) owner: #36 flags: "rxd"
    ":retrieve(STR key) => ROW LIST values";
    "  Retrieves a row stored at key";
    {raw_key} = args;
    key = this:format_key(raw_key);
    try
      return this.(key);
    except e (E_PROPNF)
      raise(E_NONE, tostr("No record found for key ", raw_key, "."));
    endtry 
  endverb

  verb "retrieve_at" (this none this) owner: #36 flags: "rxd"
    ":retrieve(STR key, INT column_index) => CELL ANY value";
    "  Retrieves a cell of row stored at key";
    {raw_key, column_index} = args;
    column_index = this:resolve_column_index(column_index);
    key = this:format_key(raw_key);
    try
      return this.(key)[column_index];
    except e (E_PROPNF)
      raise(E_NONE, tostr("No record found for key ", raw_key, "."));
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
    ":delete(OBJ/STR/FLOAT/INT key) => NONE";
    "  Deletes a key, will not throw if the key does not exist";
    {raw_key} = args;
    key = this:format_key(raw_key);
    `delete_property(this, key) ! E_PROPNF';
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

  verb "resolve_column_index" (this none this) owner: #36 flags: "rxd"
    ":resolve_column_index(STR/INT column_index) => INT";
    "  Resolves a column identifier into an index";
    "  and also validates that the index is correct.";
    {column_index} = args;
    if (typeof(column_index) == STR)
      if (are_columns_typed = $lu:is_assoc(this.columns))
        column_index = $lu:iassoc(column_index, this.columns);
      else
        column_index = column_index in this.columns;
      endif
    endif
    if (column_index <= 0 || column_index > this:column_count())
      raise(E_INVARG, tostr(column_index, " is out of bounds for the available columns."));
    endif
    return column_index;
  endverb

endobject
