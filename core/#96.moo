object #96
  name: "Diff Utils"
  parent: #78
  location: #-1
  owner: #2
  readable: true
  override "key" = 0;

  override "aliases" = {"Diff", "Utils", "Diff Utils"};

  override "help_msg" = {"STUNT DIFF UTILITY", "", "This is a repackage (01/17/2021 by Slither) of the Diff Utility & Verbs that were provided as part of the original Stunt Improvise.db. The relevant verbs were combined and rewritten to work with ToastCore. Essentially that meant rewriting how the utility outputs text, and moving from using prototype calls of :slice and :reverse to using the ToastStunt builtins for the same functionality."};

  property "default_hash_algo" (owner: #12, flags: "r") = "md5";
  
  verb "_find_common_unique_lines" (this none this) owner: #36 flags: "rxd"
    {lines1, lines2, intern} = args;
    for v, k in (intern)
      intern[k] = {{0, 0}, {0, 0}};
    endfor
    for line, i in (lines1)
      intern[line[2]][1] = {intern[line[2]][1][1] + 1, i};
    endfor
    for line, i in (lines2)
      intern[line[2]][2] = {intern[line[2]][2][1] + 1, i};
    endfor
    items = {};
    for line in (lines1)
      item = intern[line[2]];
      if (item[1][1] == 1 && item[2][1] == 1)
        items = {@items, {item[1][2], item[2][2]}};
      endif
    endfor
    return items;
  endverb

  verb "_generate_diff" (this none this) owner: #36 flags: "rxd"
    {lines1, lines2, lcs} = args;
    last = {0, 0};
    lcs = {@lcs, {length(lines1) + 1, length(lines2) + 1}};
    results = {};
    for pos in (lcs)
      i1 = last[1] + 1;
      j1 = pos[1] - 1;
      i2 = last[2] + 1;
      j2 = pos[2] - 1;
      while (i1 < j1)
        if (lines1[i1][2] == `lines2[i2][2] ! E_RANGE')
          i1 = i1 + 1;
          i2 = i2 + 1;
          continue;
        endif
        break;
      endwhile
      while (j1 >= i1)
        if (lines1[j1][2] == `lines2[j2][2] ! E_RANGE')
          j1 = j1 - 1;
          j2 = j2 - 1;
          continue;
        endif
        break;
      endwhile
      res1 = slice(lines1[i1..j1], 1);
      res2 = slice(lines2[i2..j2], 1);
      if (res1 && res2)
        results = {@results, {"r", i1, i2, res1, res2}};
      elseif (res1)
        results = {@results, {"-", i1, i2, res1}};
      elseif (res2)
        results = {@results, {"+", i1, i2, res2}};
      endif
      last = pos;
    endfor
    return results;
  endverb

  verb "_hash_lines" (this none this) owner: #36 flags: "rxd"
    {lines, ?intern = []} = args;
    algo = this.default_hash_algo;
    result = {};
    for line in (lines)
      hash = string_hash(line, algo);
      hash = `intern[hash] ! E_RANGE => intern[hash] = hash';
      result = {@result, {line, hash}};
    endfor
    return {result, intern};
  endverb

  verb "_find_lcs" (this none this) owner: #36 flags: "rxd"
    {items} = args;
    stacks = {};
    for item in (items)
      last = 0;
      for i in [1..length(stacks)]
        if (item[2] < stacks[i][$][1][2])
          stacks[i] = {@stacks[i], {item, last}};
          item = 0;
          break;
        endif
        last = length(stacks[i]);
      endfor
      if (item)
        stacks = {@stacks, {{item, last}}};
      endif
    endfor
    stacks = reverse(stacks);
    i = stacks && length(stacks[1]);
    results = {};
    for stack in (stacks)
      item = stack[i];
      results = {item[1], @results};
      i = item[2];
    endfor
    return results;
  endverb

  verb "diff" (this none this) owner: #36 flags: "rxd"
    {lines1, lines2} = args;
    intern = [];
    {lines1, intern} = this:_hash_lines(lines1, intern);
    {lines2, intern} = this:_hash_lines(lines2, intern);
    lcs = this:_find_lcs(this:_find_common_unique_lines(lines1, lines2, intern));
    return this:_generate_diff(lines1, lines2, lcs);
  endverb

  verb "diff_display" (this none this) owner: #36 flags: "rxd"
    ":diff_display(STR diff_1_name, LIST diff_1_lines, STR diff_2_name, LIST diff_2_line) => NONE";
    "this verb will diff two sets of data and display the results";
    "it is primarily designed to diff an existing verb against proposed changes";
    "the diff_1_lines are considered the primary, and the diff_2_lines are the dirty lines";
    {diff_1_name, diff_1_lines, diff_2_name, diff_2_lines} = args;
    player:tell("--- ", diff_1_name);
    player:tell("+++ ", diff_2_name, " (dirty)");
    diffs = this:diff(diff_1_lines, diff_2_lines);
    for diff in (diffs)
      pos1 = diff[2];
      pos2 = diff[3];
      if (diff[1] == "r")
        df1 = diff[4];
        ln1 = length(df1);
        df2 = diff[5];
        ln2 = length(df2);
      elseif (diff[1] == "+")
        df1 = {};
        ln1 = 0;
        df2 = diff[4];
        ln2 = length(df2);
      elseif (diff[1] = "-")
        df1 = diff[4];
        ln1 = length(df1);
        df2 = {};
        ln2 = 0;
      endif
      before = pos1 > 1 ? tostr(" ", diff_1_lines[pos1 - 1]) | 0;
      after = pos1 + ln1 - 1 < length(diff_1_lines) ? tostr(" ", diff_1_lines[pos1 + ln1]) | 0;
      player:tell("", "@@ -", pos1, ",", ln1, " +", pos2, ",", ln2, " @@");
      before && player:notify(before);
      for l in (df1)
        player:tell($ansi:red(tostr("-", l)));
      endfor
      for l in (df2)
        player:tell($ansi:green(tostr("+", l)));
      endfor
      after && player:tell(after);
    endfor
    player:tell("(done)");
  endverb

endobject
