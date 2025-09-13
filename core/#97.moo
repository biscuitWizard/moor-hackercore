
object #97
  name: "Help Datastore"
  parent: #38
  location: #84
  owner: #36
  readable: true

  property "help_categories" (owner: #36, flags: "r") = {"general", "admin", "prog", "builder"};

  verb "permitted_categories" (this none this) owner: #36 flags: "rxd"
    ":permitted_categories(OBJ dude) => LIST categories dude can see";
    {dude} = args;
    cats = {"general"};
    if (!$wiz_utils:is_admin(dude))
      return cats;
    endif
    if (dude.programmer)
      cats = setadd(cats, "prog");
    endif
    if ($ou:isa(dude, $builder))
      cats = setadd(cats, "builder");
    endif
    return cats;
  endverb

  verb "find" (this none this) owner: #12 flags: "rxd"
    ":find(STR keyword[, LIST categories]) => LIST of results";
    {keyword, ?categories = {"general"}} = args;
    query = "SELECT hf.category, hf.content, i.keyword FROM help_file_index AS i LEFT JOIN help_file AS hf ON hf.id=i.help_file_id WHERE i.keyword LIKE $1 AND is_deleted = 0 AND hf.category in (";
    for cat in (categories)
      query = query + "'" + cat + "',";
    endfor
    query = $su:trim(query, ",") + ")";
    results = $sql:query(this:connection_id(), query, {$su:subst(keyword, {{"*", "%"}})});
    for id in [1..length(results)]
      results[id][2] = parse_json(results[id][2]);
    endfor
    if (!results && !("*" in keyword))
      return this:find(tostr(keyword, "*"), categories);
    endif
    return results;
  endverb
endobject
