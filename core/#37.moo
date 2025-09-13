object #37
  name: "Generic Columnar Datastore"
  parent: #17
  location: #84
  owner: #36
  readable: true

  property "columns" (owner: #36, flags: "r") = {};

  verb "save" (this none this) owner: #36 flags: "rxd"
    ":save(OBJ/STR/INT/FLOAT key, LIST values) => NONE";
    "  Adds or updates a value in the datastore";
    $raise_not_implemented();
  endverb

  verb "save_at" (this none this) owner: #36 flags: "rxd"
    ":save_at(OBJ/STR/INT/FLOAT key, INT column_index, ANY value) => ROW LIST values";
    "  Saves a value at a specific cell.";
    "  column index is 1-based, not 0-based.";
    $raise_not_implemented();
  endverb

  verb "retrieve" (this none this) owner: #36 flags: "rxd"
    ":retrieve(OBJ/STR/INT/FLOAT key) => ROW LIST values";
    "  Retrieves a row stored at key";
    $raise_not_implemented();
  endverb

  verb "retrieve_at" (this none this) owner: #36 flags: "rxd"
    ":retrieve(OBJ/STR/INT/FLOAT key, INT column_index) => CELL ANY value";
    "  Retrieves a cell of row stored at key";
    $raise_not_implemented();
  endverb

  verb "keys" (this none this) owner: #36 flags: "rxd"
    ":keys() => LIST of all keys present on datastore";
    $raise_not_implemented();
  endverb

  verb "delete" (this none this) owner: #36 flags: "rxd"
    ":delete(OBJ/STR/FLOAT/INT key) => NONE";
    "  Deletes a key, will not throw if the key does not exist";
    $raise_not_implemented();
  endverb

  verb "has_key" (this none this) owner: #36 flags: "rxd"
    ":has(OBJ/STR/FLOAT/INT key) => BOOL if key exits";
    "  retrieves data located at key";
    $raise_not_implemented();
  endverb

  verb "validate_row" (this none this) owner: #36 flags: "rxd"
    ":validate_row(LIST data) => NONE";
    "  will throw loudly if the data isn't in a good format";
    {data} = args;
    callers = callers();
    "check the size of the data matches our columns";
    if (length(data) != this:column_count())
      raise(E_INVARG, tostr(callers[1][1], ":", callers[1][2], " was expecting ", length(this.columns), " columns worth of data but got ", length(data), "."));
    endif
    if ($lu:is_assoc(data))
      "we can check the TYPES of each of the data for enforcement.";
      for cell, i in (data)
        valid_types = typeof(this.columns[i][2]) == LIST ? this.columns[i][2] | {this.columns[i][2]};
        if (typeof(cell) in valid_types)
          continue;
        endif
        raise(E_INVARG, tostr(callers[1][1], ":", callers[1][2], " was expecting ", $su:english_list(valid_types), " TYPE but got ", typeof(cell), "."));
      endfor
    endif
  endverb
  
  verb "column_count" (this none this) owner: #36 flags: "rxd"
    ":column_count() => INT number of columns";
    return length(this.columns);
  endverb

  verb "@migrate" (this to any) owner: #2 flags: "rd"
    
  endverb

endobject
