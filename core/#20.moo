object #20
  name: "String Utilities"
  parent: #78
  location: #-1
  owner: #2
  readable: true
  override "aliases" = {"String Utilities"};

  override "description" = {"This is the string utilities utility package.  See `help $string_utils' for more details."};

  override "help_msg" = {"For a complete description of a given verb, do `help $string_utils:verbname'", "", "    Conversion routines:", "", ":from_list    (list [,sep])                          => \"foo1foo2foo3\"", ":english_list (str-list[,none-str[,and-str[, sep]]]) => \"foo1, foo2, and foo3\"", ":title_list*c (obj-list[,none-str[,and-str[, sep]]]) => \"foo1, foo2, and foo3\"", "                                                  or => \"Foo1, foo2, and foo3\"", ":from_value   (value [,quoteflag [,maxlistdepth]])   => \"{foo1, foo2, foo3}\"", ":print        (value)                                => value in string", ":abbreviated_value (value, options)                  => short value in string", "", ":to_value       (string)     => {success?, value or error message}", ":prefix_to_value(string)     => {rest of string, value} or {0, error message}", "", ":english_number(42)          => \"forty-two\"", ":english_ordinal(42)         => \"forty-second\"", ":ordinal(42)                 => \"42nd\"", ":group_number(42135 [,sep])  => \"42,135\"", ":from_ASCII(65)              => \"A\"", ":to_ASCII(\"A\")               => 65", ":from_seconds(number)        => string of rough time passed in large increments", "", ":name_and_number(obj [,sep]) => \"ObjectName (#obj)\"", ":name_and_number_list({obj1,obj2} [,sep])", "                             => \"ObjectName1 (#obj1) and ObjectName2 (#obj2)\"", ":nn is an alias for :name_and_number.", ":nn_list is an alias for :name_and_number_list.", "", "    Type checking:", "", ":is_integer   (string) => return true if string is composed entirely of digits", ":is_float     (string) => return true if string holds just a floating point", "", "    Parsing:", "", ":explode (string,char) -- string => list of words delimited by char", ":words   (string)      -- string => list of words (as with command line parser)", ":word_start (string)   -- string => list of start-end pairs.", ":first_word (string)   -- string => list {first word, rest of string} or {}", ":char_list  (string)   -- string => list of characters in string", "", ":parse_command (cmd_line [,player] => mimics action of builtin parser", "", "    Matching:", "", ":find_prefix  (prefix, string-list)=>list index of element starting with prefix", ":index_delimited(string,target[,case]) =>index of delimited string occurrence", ":index_all    (string, target string)          => list of all matched positions", ":common       (first string, second string)  => length of longest common prefix", ":match        (string, [obj-list, prop-name]+) => matching object", ":match_player (string-list[,me-object])        => list of matching players", ":match_object (string, location)               => default object match...", ":match_player_or_object (string, location) => object then player matching", ":literal_object (string)                       => match against #xxx, $foo", ":match_stringlist (string, targets)            => match against static strings", ":match_string (string, wildcard target,options)=> match against a wildcard", "", "    Pretty printing:", "", ":space         (n/string[,filler])     => n spaces", ":left          (string,width[,filler]) => left justified string in field ", ":right         (string,width[,filler]) => right justified string in field", ":center/re     (string,width[,lfiller[,rfiller]]) => centered string in field", ":columnize/se  (list,n[,width])        => list of strings in n columns", "", "    Substitutions", "", ":substitute (string,subst_list [,case])   -- general substitutions.", ":substitute_delimited (string,subst_list [,case])", "                                          -- like subst, but uses index_delim", ":pronoun_sub (string/list[,who[,thing[,location]]])", "                                          -- pronoun substitutions.", ":pronoun_sub_secure (string[,who[,thing[,location]]],default)", "                                          -- substitute and check for names.", ":pronoun_quote (string/list/subst_list)   -- quoting for pronoun substitutions.", "", "    Miscellaneous string munging:", "", ":trim         (string)       => string with outside whitespace removed.", ":triml        (string)       => string with leading whitespace removed.", ":trimr        (string)       => string with trailing whitespace removed.", ":strip_chars  (string,chars) => string with all chars in `chars' removed.", ":strip_all_but(string,chars) => string with all chars not in `chars' removed.", ":capitalize/se(string)       => string with first letter capitalized.", ":uppercase/lowercase(string) => string with all letters upper or lowercase.", ":names_of     (list of OBJ)  => string with names and object numbers of items.", ":a_or_an      (word)         => \"a\" or \"an\" as appropriate for that word.", ":reverse      (string)       => \"gnirts\"", ":incr_alpha   (string)       => \"increments\" the string alphabetically", "", "    A useful property:", "", ".alphabet                    => \"abcdefghijklmnopqrstuvwxyz\""};

  property "digits" (owner: #2, flags: "rc") = "0123456789";

  property "ascii" (owner: #2, flags: "rc") = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

  property "alphabet" (owner: #2, flags: "rc") = "abcdefghijklmnopqrstuvwxyz";

  property "use_article_a" (owner: #36, flags: "r") = {"unit", "unix", "one", "once", "utility"};

  property "use_article_an" (owner: #36, flags: "r") = {};

  property "tab" (owner: #2, flags: "rc") = "	";

  property "vowels" (owner: #2, flags: "r") = {"a", "o", "u", "i", "e"};

  property "SINGULARIZATION_RULES" (owner: #2, flags: "r") = ["(agend|addend|millenni|dat|extrem|bacteri|desiderat|strat|candelabr|errat|ov|symposi|curricul|quor)a$" -> "$1um", "(alumn|alg|vertebr)ae$" -> "$1a", "(alumn|syllab|vir|radi|nucle|fung|cact|stimul|termin|bacill|foc|uter|loc|strat)(?:us|i)$" -> "$1us", "(analy|diagno|parenthe|progno|synop|the|empha|cri|ne)(?:sis|ses)$" -> "$1sis", "(apheli|hyperbat|periheli|asyndet|noumen|phenomen|criteri|organ|prolegomen|hedr|automat)a$" -> "$1on", "(ar|(?:wo|[ae])l|[eo][ao])ves$" -> "$1f", "(child)ren$" -> "$1", "(cod|mur|sil|vert|ind)ices$" -> "$1ex", "(dg|ss|ois|lk|ok|wn|mb|th|ch|ec|oal|is|ck|ix|sser|ts|wb)ies$" -> "$1ie", "(eau)x?$" -> "$1", "(matr|append)ices$" -> "$1ix", "(movie|twelve|abuse|e[mn]u)s$" -> "$1", "(pe)(rson|ople)$" -> "$1rson", "(seraph|cherub)im$" -> "$1", "(ss)$" -> "$1", "(test)(?:is|es)$" -> "$1is", "(wi|kni|(?:after|half|high|low|mid|non|night|[^w]|^)li)ves$" -> "$1fe", "(x|ch|ss|sh|zz|tto|go|cho|alias|[^aou]us|t[lm]as|gas|(?:her|at|gr)o|[aeiou]ris)(?:es)?$" -> "$1", "b((?:tit)?m|l)ice$" -> "$1ouse", "b(l|(?:neck|cross|hog|aun)?t|coll|faer|food|gen|goon|group|hipp|junk|vegg|(?:pork)?p|charl|calor|cut)ies$" -> "$1ie", "b(mon|smil)ies$" -> "$1ey", "ies$" -> "y", "men$" -> "man", "s$" -> ""];

  property "IRREGULAR_RULES" (owner: #2, flags: "r") = ["anathema" -> "anathemata", "axe" -> "axes", "canvas" -> "canvases", "carve" -> "carves", "die" -> "dice", "dingo" -> "dingoes", "dogma" -> "dogmata", "eave" -> "eaves", "echo" -> "echoes", "foot" -> "feet", "genus" -> "genera", "goose" -> "geese", "groove" -> "grooves", "has" -> "have", "he" -> "they", "her" -> "their", "herself" -> "themselves", "himself" -> "themselves", "his" -> "their", "human" -> "humans", "I" -> "we", "is" -> "are", "its" -> "their", "itself" -> "themselves", "lemma" -> "lemmata", "looey" -> "looies", "me" -> "us", "my" -> "our", "myself" -> "ourselves", "ox" -> "oxen", "passerby" -> "passersby", "pickaxe" -> "pickaxes", "proof" -> "proofs", "quiz" -> "quizzes", "schema" -> "schemata", "she" -> "they", "stigma" -> "stigmata", "stoma" -> "stomata", "that" -> "those", "them" -> "them", "themself" -> "themselves", "thief" -> "thieves", "this" -> "these", "tooth" -> "teeth", "tornado" -> "tornadoes", "torpedo" -> "torpedoes", "valve" -> "valves", "viscus" -> "viscera", "volcano" -> "volcanoes", "was" -> "were", "yes" -> "yeses", "yourself" -> "yourselves"];

  property "UNCOUNTABLE_RULES" (owner: #2, flags: "r") = {"adulthood", "advice", "agenda", "aid", "aircraft", "alcohol", "ammo", "analytics", "anime", "athletics", "audio", "bison", "blood", "bream", "buffalo", "butter", "carp", "cash", "chassis", "chess", "clothing", "cod", "commerce", "cooperation", "corps", "debris", "diabetes", "digestion", "elk", "energy", "equipment", "excretion", "expertise", "firmware", "flounder", "fun", "gallows", "garbage", "graffiti", "hardware", "headquarters", "health", "herpes", "highjinks", "homework", "housework", "information", "jeans", "justice", "kudos", "labour", "literature", "machinery", "mackerel", "mail", "media", "mews", "moose", "music", "mud", "manga", "news", "only", "personnel", "pike", "plankton", "pliers", "police", "pollution", "premises", "rain", "research", "rice", "salmon", "scissors", "series", "sewage", "shambles", "shrimp", "software", "staff", "swine", "tennis", "traffic", "transportation", "trout", "tuna", "wealth", "welfare", "whiting", "wildebeest", "wildlife", "you", "pok[e]mon$", "[^aeiou]ese$", "deer$", "fish$", "measles$", "o[iu]s$", "pox$", "sheep$"};

  property "PLURALIZATION_RULES" (owner: #2, flags: "r") = ["(?:(kni|wi|li)fe|(ar|l|ea|eo|oa|hoo)f)$" -> "$1$2ves", "([^aeiou]ese)$" -> "$1", "([^aeiouy]|qu)y$" -> "$1ies", "([^ch][ieo][ln])ey$" -> "$1ies", "([^l]ias|[aeiou]las|[ejzr]as|[iu]am)$" -> "$1", "(agend|addend|millenni|dat|extrem|bacteri|desiderat|strat|candelabr|errat|ov|symposi|curricul|automat|quor)(?:a|um)$" -> "$1a", "(alias|[^aou]us|t[lm]as|gas|ris)$" -> "$1es", "(alumn|alg|vertebr)(?:a|ae)$" -> "$1ae", "(alumn|syllab|vir|radi|nucle|fung|cact|stimul|termin|bacill|foc|uter|loc|strat)(?:us|i)$" -> "$1i", "(apheli|hyperbat|periheli|asyndet|noumen|phenomen|criteri|organ|prolegomen|hedr|automat)(?:a|on)$" -> "$1a", "(ax|test)is$" -> "$1es", "(child)(?:ren)?$" -> "$1ren", "(e[mn]u)s?$" -> "$1s", "(her|at|gr)o$" -> "$1oes", "(matr|cod|mur|sil|vert|ind|append)(?:ix|ex)$" -> "$1ices", "(pe)(?:rson|ople)$" -> "$1ople", "(seraph|cherub)(?:im)?$" -> "$1im", "(x|ch|ss|sh|zz)$" -> "$1es", "[^u0000-u007F]$" -> "$0", "b((?:tit)?m|l)(?:ice|ouse)$" -> "$1ice", "eaux$" -> "$0", "m[ae]n$" -> "men", "s?$" -> "s", "sisi" -> "ses", "thou" -> "you"];

  property "IRREGULAR_PLURAL_RULES" (owner: #2, flags: "r") = ["anathemata" -> "anathema", "are" -> "is", "axes" -> "axe", "canvases" -> "canvas", "carves" -> "carve", "dice" -> "die", "dingoes" -> "dingo", "dogmata" -> "dogma", "eaves" -> "eave", "echoes" -> "echo", "feet" -> "foot", "geese" -> "goose", "genera" -> "genus", "goes" -> "go", "grooves" -> "groove", "have" -> "has", "humans" -> "human", "lemmata" -> "lemma", "looies" -> "looey", "our" -> "my", "ourselves" -> "myself", "oxen" -> "ox", "passersby" -> "passerby", "pickaxes" -> "pickaxe", "proofs" -> "proof", "quizzes" -> "quiz", "schemata" -> "schema", "stigmata" -> "stigma", "stomata" -> "stoma", "teeth" -> "tooth", "their" -> "its", "them" -> "them", "themselves" -> "themself", "these" -> "this", "they" -> "she", "thieves" -> "thief", "those" -> "that", "tornadoes" -> "tornado", "torpedoes" -> "torpedo", "us" -> "me", "valves" -> "valve", "viscera" -> "viscus", "volcanoes" -> "volcano", "we" -> "I", "were" -> "was", "yeses" -> "yes", "yourselves" -> "yourself"];

  property "IRREGULAR_SINGULAR_RULES" (owner: #2, flags: "r") = ["anathema" -> "anathemata", "axe" -> "axes", "canvas" -> "canvases", "carve" -> "carves", "die" -> "dice", "dingo" -> "dingoes", "dogma" -> "dogmata", "eave" -> "eaves", "echo" -> "echoes", "foot" -> "feet", "genus" -> "genera", "go" -> "goes", "goose" -> "geese", "groove" -> "grooves", "has" -> "have", "he" -> "they", "her" -> "their", "herself" -> "themselves", "himself" -> "themselves", "his" -> "their", "human" -> "humans", "I" -> "we", "is" -> "are", "its" -> "their", "itself" -> "themselves", "lemma" -> "lemmata", "looey" -> "looies", "me" -> "us", "my" -> "our", "myself" -> "ourselves", "ox" -> "oxen", "passerby" -> "passersby", "pickaxe" -> "pickaxes", "proof" -> "proofs", "quiz" -> "quizzes", "schema" -> "schemata", "she" -> "they", "stigma" -> "stigmata", "stoma" -> "stomata", "that" -> "those", "them" -> "them", "themself" -> "themselves", "thief" -> "thieves", "this" -> "these", "tooth" -> "teeth", "tornado" -> "tornadoes", "torpedo" -> "torpedoes", "valve" -> "valves", "viscus" -> "viscera", "volcano" -> "volcanoes", "was" -> "were", "yes" -> "yeses", "yourself" -> "yourselves"];

  property "exit_directions" (owner: #2, flags: "r") = ["a" -> "aft", "d" -> "down", "e" -> "east", "f" -> "fore", "n" -> "north", "ne" -> "northeast", "nw" -> "northwest", "p" -> "port", "po" -> "port", "s" -> "south", "se" -> "southeast", "st" -> "starboard", "star" -> "starboard", "sw" -> "southwest", "u" -> "up", "w" -> "west"];

  verb "space(noansi)" (this none this) owner: #36 flags: "rxd"
    "space(len,fill) returns a string of length abs(len) consisting of copies of fill.  If len is negative, fill is anchored on the right instead of the left.";
    {n, ?fill = " "} = args;
    if (typeof(n) == STR)
      n = length(n);
    endif
    if (n > 1000)
      "Prevent someone from crashing the moo with $string_utils:space($maxint)";
      return E_INVARG;
    endif
    if (" " != fill)
      fill = fill + fill;
      fill = fill + fill;
      fill = fill + fill;
    elseif ((n = abs(n)) < 70)
      return "                                                                      "[1..n];
    else
      fill = "                                                                      ";
    endif
    m = (n - 1) / length(fill);
    while (m)
      fill = fill + fill;
      m = m / 2;
    endwhile
    return n > 0 ? fill[1..n] | fill[$ + 1 + n..$];
  endverb

  verb "left(noansi)" (this none this) owner: #36 flags: "rxd"
    "$string_utils:left(string,width[,filler])";
    "";
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> followed by enough filler to make it that wide. If <width> is negative and the length of <string> is greater than the absolute value of <width>, then the <string> is cut off at <width>.";
    "";
    "The <filler> is optional and defaults to \" \"; it controls what is used to fill the resulting string when it is too short.  The <filler> is replicated as many times as is necessary to fill the space in question.";
    {text, len, ?fill = " "} = args;
    abslen = abs(len);
    out = tostr(text);
    if (length(out) < abslen)
      return out + this:space(length(out) - abslen, fill);
    else
      return len > 0 ? out | out[1..abslen];
    endif
  endverb

  verb "right(noansi)" (this none this) owner: #36 flags: "rxd"
    "$string_utils:right(string,width[,filler])";
    "";
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> preceded by enough filler to make it that wide. If <width> is negative and the length of <string> is greater than the absolute value of <width>, then <string> is cut off at <width> from the right.";
    "";
    "The <filler> is optional and defaults to \" \"; it controls what is used to fill the resulting string when it is too short.  The <filler> is replicated as many times as is necessary to fill the space in question.";
    {text, len, ?fill = " "} = args;
    abslen = abs(len);
    out = tostr(text);
    if ((lenout = length(out)) < abslen)
      return this:space(abslen - lenout, fill) + out;
    else
      return len > 0 ? out | out[$ - abslen + 1..$];
    endif
  endverb

  verb "centre center" (this none this) owner: #36 flags: "rxd"
    "$string_utils:center(string,width[,lfiller[,rfiller]])";
    "";
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> preceded and followed by enough filler to make it that wide.  If <width> is negative and the length of <string> is greater than the absolute value of <width>, then the <string> is cut off at <width>.";
    "";
    "The <lfiller> is optional and defaults to \" \"; it controls what is used to fill the left part of the resulting string when it is too short.  The <rfiller> is optional and defaults to the value of <lfiller>; it controls what is used to fill the right part of the resulting string when it is too short.  In both cases, the filler is replicated as many times as is necessary to fill the space in question.";
    {text, len, ?lfill = " ", ?rfill = lfill} = args;
    out = tostr(text);
    abslen = abs(len);
    if (length(out) < abslen)
      return this:space((abslen - length(out)) / 2, lfill) + out + this:space((abslen - length(out) + 1) / -2, rfill);
    else
      return len > 0 ? out | out[1..abslen];
    endif
  endverb

  verb "columnize columnise" (this none this) owner: #36 flags: "rxd"
    "columnize (items, n [, width]) - Turn a one-column list of items into an n-column list. 'width' is the last character position that may be occupied; it defaults to a standard screen width. Example: To tell the player a list of numbers in three columns, do 'player:tell_lines ($string_utils:columnize ({1, 2, 3, 4, 5, 6, 7}, 3));'.";
    {items, n, ?width = 79} = args;
    height = (length(items) + n - 1) / n;
    items = {@items, @$list_utils:make(height * n - length(items), "")};
    colwidths = {};
    for col in [1..n - 1]
      colwidths = listappend(colwidths, 1 - (width + 1) * col / n);
    endfor
    result = {};
    for row in [1..height]
      line = tostr(items[row]);
      for col in [1..n - 1]
        line = tostr(this:left(line, colwidths[col]), " ", items[row + col * height]);
      endfor
      result = listappend(result, line[1..min($, width)]);
    endfor
    return result;
  endverb

  verb "from_list" (this none this) owner: #36 flags: "rxd"
    "$string_utils:from_list(list [, separator])";
    "Return a string being the concatenation of the string representations of the elements of LIST, each pair separated by the string SEPARATOR, which defaults to the empty string.";
    {thelist, ?separator = ""} = args;
    if (separator == "")
      return tostr(@thelist);
    elseif (thelist)
      result = tostr(thelist[1]);
      for elt in (listdelete(thelist, 1))
        result = tostr(result, separator, elt);
      endfor
      return result;
    else
      return "";
    endif
  endverb

  verb "english_list" (this none this) owner: #36 flags: "rxd"
    "Prints the argument (must be a list) as an english list, e.g. {1, 2, 3} is printed as \"1, 2, and 3\", and {1, 2} is printed as \"1 and 2\".";
    "Optional arguments are treated as follows:";
    "  Second argument is the string to use when the empty list is given.  The default is \"nothing\".";
    "  Third argument is the string to use in place of \" and \".  A typical application might be to use \" or \" instead.";
    "  Fourth argument is the string to use instead of a comma (and space).  Gary_Severn's deranged mind actually came up with an application for this.  You can ask him.";
    "  Fifth argument is a string to use after the penultimate element before the \" and \".  The default is to have a comma without a space.";
    {things, ?nothingstr = "nothing", ?andstr = " and ", ?commastr = ", ", ?finalcommastr = ","} = args;
    nthings = length(things);
    if (nthings == 0)
      return nothingstr;
    elseif (nthings == 1)
      return tostr(things[1]);
    elseif (nthings == 2)
      return tostr(things[1], andstr, things[2]);
    else
      ret = "";
      for k in [1..nthings - 1]
        if (k == nthings - 1)
          commastr = finalcommastr;
        endif
        ret = tostr(ret, things[k], commastr);
      endfor
      return tostr(ret, andstr, things[nthings]);
    endif
  endverb

  verb "names_of" (this none this) owner: #36 flags: "rxd"
    "Return a string of the names and object numbers of the objects in a list.";
    line = "";
    for item in (args[1])
      if (typeof(item) == OBJ && valid(item))
        line = line + item.name + "(" + tostr(item) + ")   ";
      endif
    endfor
    return $string_utils:trimr(line);
  endverb

  verb "from_seconds" (this none this) owner: #36 flags: "rxd"
    ":from_seconds(number of seconds) => returns a string containing the rough increment of days, or hours if less than a day, or minutes if less than an hour, or lastly in seconds.";
    ":from_seconds(86400) => \"a day\"";
    ":from_seconds(7200)  => \"two hours\"";
    minute = 60;
    hour = 60 * minute;
    day = 24 * hour;
    secs = args[1];
    if (secs > day)
      count = secs / day;
      unit = "day";
      article = "a";
    elseif (secs > hour)
      count = secs / hour;
      unit = "hour";
      article = "an";
    elseif (secs > minute)
      count = secs / minute;
      unit = "minute";
      article = "a";
    else
      count = secs;
      unit = "second";
      article = "a";
    endif
    if (count == 1)
      time = tostr(article, " ", unit);
    else
      time = tostr(count, " ", unit, "s");
    endif
    return time;
  endverb

  verb "trim" (this none this) owner: #36 flags: "rxd"
    ":trim (string [, space]) -- remove leading and trailing spaces";
    "";
    "`space' should be a character (single-character string); it defaults to \" \".  Returns a copy of string with all leading and trailing copies of that character removed.  For example, $string_utils:trim(\"***foo***\", \"*\") => \"foo\".";
    {string, ?space = " "} = args;
    m = match(string, tostr("[^", space, "]%(.*[^", space, "]%)?%|$"));
    return string[m[1]..m[2]];
  endverb

  verb "triml" (this none this) owner: #36 flags: "rxd"
    ":triml(string [, space]) -- remove leading spaces";
    "";
    "`space' should be a character (single-character string); it defaults to \" \".  Returns a copy of string with all leading copies of that character removed.  For example, $string_utils:triml(\"***foo***\", \"*\") => \"foo***\".";
    {string, ?what = " "} = args;
    m = match(string, tostr("[^", what, "]%|$"));
    return string[m[1]..$];
  endverb

  verb "trimr" (this none this) owner: #36 flags: "rxd"
    ":trimr(string [, space]) -- remove trailing spaces";
    "";
    "`space' should be a character (single-character string); it defaults to \" \".  Returns a copy of string with all trailing copies of that character removed.  For example, $string_utils:trimr(\"***foo***\", \"*\") => \"***foo\".";
    {string, ?what = " "} = args;
    return string[1..rmatch(string, tostr("[^", what, "]%|^"))[2]];
  endverb

  verb "strip_chars" (this none this) owner: #36 flags: "rxd"
    ":strip_chars(string,chars) => string with chars removed";
    {subject, stripped} = args;
    for i in [1..length(stripped)]
      subject = strsub(subject, stripped[i], "");
    endfor
    return subject;
  endverb

  verb "strip_all_but" (this none this) owner: #36 flags: "rxd"
    ":strip_all_but(string,keep) => string with chars not in `keep' removed.";
    "`keep' is used in match() so if it includes ], ^, or -,";
    "] should be first, ^ should be other from first, and - should be last.";
    string = args[1];
    wanted = "[" + args[2] + "]+";
    output = "";
    while (m = match(string, wanted))
      output = output + string[m[1]..m[2]];
      string = string[m[2] + 1..$];
    endwhile
    return output;
  endverb

  verb "uppercase lowercase uc lc" (this none this) owner: #36 flags: "rxd"
    "lowercase(string) -- returns a lowercase version of the string.";
    "uppercase(string) -- returns the uppercase version of the string.";
    string = args[1];
    from = caps = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    to = lower = "abcdefghijklmnopqrstuvwxyz";
    if (verb == "uppercase" || verb == "uc")
      from = lower;
      to = caps;
    endif
    for i in [1..26]
      string = strsub(string, from[i], to[i], 1);
    endfor
    return string;
  endverb

  verb "capitalize capitalise" (this none this) owner: #36 flags: "rxd"
    "capitalizes its argument.";
    if ((string = args[1]) && (i = index("abcdefghijklmnopqrstuvwxyz", string[1], 1)))
      string[1] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i];
    endif
    return string;
  endverb

  verb "literal_object" (this none this) owner: #36 flags: "rxd"
    "Matches args[1] against literal objects: #xxxxx, $variables, *mailing-lists, and username.  Returns the object if successful, $failed_match else.";
    string = args[1];
    if (!string)
      return $nothing;
    elseif (string[1] == "#" && E_TYPE != (object = $code_utils:toobj(string)))
      return object;
    elseif (string[1] == "~")
      return this:match_player(string[2..$], #0);
    elseif (string[1] == "*" && length(string) > 1)
      return $mail_agent:match_recipient(string);
    elseif (string[1] == "$")
      string[1..1] = "";
      object = #0;
      while (pn = string[1..(dot = index(string, ".")) ? dot - 1 | $])
        if (!$object_utils:has_property(object, pn) || typeof(object = object.(pn)) != OBJ)
          "Try to match a map now.";
          object = $code_utils:parse_sysobj_map(args[1]);
          if (object == E_PROPNF)
            return $failed_match;
          else
            break;
          endif
        endif
        string = string[length(pn) + 2..$];
      endwhile
      if (object == #0 || typeof(object) == ERR)
        return $failed_match;
      else
        return object;
      endif
    else
      return $failed_match;
    endif
  endverb

  verb "match" (this none this) owner: #36 flags: "rxd"
    "$string_utils:match(string [, obj-list, prop-name]*)";
    "Each obj-list should be a list of objects or a single object, which is treated as if it were a list of that object.  Each prop-name should be string naming a property on every object in the corresponding obj-list.  The value of that property in each case should be either a string or a list of strings.";
    "The argument string is matched against all of the strings in the property values.";
    "If it exactly matches exactly one of them, the object containing that property is returned.  If it exactly matches more than one of them, $ambiguous_match is returned.";
    "If there are no exact matches, then partial matches are considered, ones in which the given string is a prefix of some property string.  Again, if exactly one match is found, the object with that property is returned, and if there is more than one match, $ambiguous_match is returned.";
    "Finally, if there are no exact or partial matches, then $failed_match is returned.";
    subject = args[1];
    if (subject == "")
      return $nothing;
    endif
    no_exact_match = no_partial_match = 1;
    for i in [1..length(args) / 2]
      prop_name = args[2 * i + 1];
      for object in (typeof(olist = args[2 * i]) == LIST ? olist | {olist})
        if (`valid(object) ! ANY => #-1')
          if (typeof(str_list = `object:(prop_name)() ! E_PERM, E_VERBNF => `object.(prop_name) ! E_PERM, E_PROPNF => {}'') != LIST)
            str_list = {str_list};
          endif
          if (subject in str_list)
            if (no_exact_match)
              no_exact_match = object;
            elseif (no_exact_match != object)
              return $ambiguous_match;
            endif
          else
            for string in (str_list)
              if (index(string, subject) != 1)
              elseif (no_partial_match)
                no_partial_match = object;
              elseif (no_partial_match != object)
                no_partial_match = $ambiguous_match;
              endif
            endfor
          endif
        endif
      endfor
    endfor
    return no_exact_match && (no_partial_match && $failed_match);
  endverb

  verb "match_str*ing" (this none this) owner: #36 flags: "rxd"
    "* wildcard matching. Returns a list of what the *s actually matched. Won't cath every match, if there are several ways to parse it.";
    "Example: $string_utils:match_string(\"Jack waves to Jill\",\"* waves to *\") returns {\"Jack\", \"Jill\"}";
    "Optional arguments: numbers are interpreted as case-sensitivity, strings as alternative wildcards.";
    {what, targ, @rest} = args;
    wild = "*";
    case = ret = {};
    what = what + "&^%$";
    targ = targ + "&^%$";
    for y in (rest)
      if (typeof(y) == STR)
        wild = y;
      elseif (typeof(y) == INT)
        case = {y};
      endif
    endfor
    while (targ != "")
      if (z = index(targ, wild))
        part = targ[1..z - 1];
      else
        z = length(targ);
        part = targ;
      endif
      n = part == "" ? 1 | index(what, part, @case);
      if (n)
        ret = listappend(ret, what[1..n - 1]);
        what = what[z + n - 1..$];
        targ = targ[z + 1..$];
      else
        return 0;
      endif
    endwhile
    if (ret == {})
      return what == "";
    elseif (ret == {""})
      return 1;
    elseif (ret[1] == "")
      return ret[2..$];
    else
      return 0;
    endif
  endverb

  verb "match_player" (this none this) owner: #36 flags: "rxd"
    "match_player(name,name,...)      => {obj,obj,...}";
    "match_player(name[,meobj])       => obj";
    "match_player({name,...}[,meobj]) => {obj,...}";
    "objs returned are either players, $failed_match, $ambiguous_match, or $nothing in the case of an empty string.";
    "meobj (what to return for instances of `me') defaults to player; if given and isn't actually a player, `me' => $failed_match";
    retstr = 0;
    me = player;
    if (length(args) < 2 || typeof(me = args[2]) == OBJ)
      me = valid(me) && is_player(me) ? me | $failed_match;
      if (typeof(args[1]) == STR)
        strings = {args[1]};
        retstr = 1;
        "return a string, not a list";
      else
        strings = args[1];
      endif
    else
      strings = args;
      me = player;
    endif
    found = {};
    for astr in (strings)
      if (!astr)
        aobj = $nothing;
      elseif (astr == "me")
        aobj = me;
      elseif (valid(aobj = $string_utils:literal_object(astr)) && is_player(aobj))
        "astr is a valid literal object number of some player, so we are done.";
      else
        matches = complex_match(astr, players());
        aobj = typeof(matches) == LIST ? matches[1] | matches;
      endif
      found = {@found, aobj};
    endfor
    return retstr ? found[1] | found;
  endverb

  verb "match_player_or_object" (this none this) owner: #36 flags: "rxd"
    "Accepts any number of strings, attempts to match those strings first against objects in the room, and if no objects by those names exist, matches against player names (and \"#xxxx\" style strings regardless of location).  Returns a list of valid objects so found.";
    "Unlike $string_utils:match_player, does not include in the list the failed and ambiguous matches; instead has built-in error messages for such objects.  This should probably be improved.  Volunteers?";
    if (!args)
      return;
    endif
    unknowns = {};
    objs = {};
    "We have to do something icky here.  Parallel walk the victims and args lists.  When it's a valid object, then it's a player.  If it's an invalid object, try to get an object match from the room.  If *that* fails, complain.";
    for i in [1..length(args)]
      if (valid(o = player.location:match(args[i])))
        objs = {@objs, o};
      else
        unknowns = {@unknowns, args[i]};
      endif
    endfor
    victims = $string_utils:match_player(unknowns);
    for i in [1..length(victims)]
      if (valid(victims[i]))
        objs = {@objs, victims[i]};
      endif
    endfor
    return objs;
  endverb

  verb "find_prefix" (this none this) owner: #36 flags: "rxd"
    "find_prefix(prefix, string-list) => list index of something starting with prefix, or 0 or $ambiguous_match.";
    {subject, choices} = args;
    answer = 0;
    for i in [1..length(choices)]
      if (index(choices[i], subject) == 1)
        if (answer == 0)
          answer = i;
        else
          answer = $ambiguous_match;
        endif
      endif
    endfor
    return answer;
  endverb

  verb "index_d*elimited" (this none this) owner: #36 flags: "rxd"
    "index_delimited(string,target[,case_matters]) is just like the corresponding call to the builtin index() but instead only matches on occurences of target delimited by word boundaries (i.e., not preceded or followed by an alphanumeric)";
    args[2] = "%(%W%|^%)" + $string_utils:regexp_quote(args[2]) + "%(%W%|$%)";
    return (m = match(@args)) ? m[3][1][2] + 1 | 0;
  endverb

  verb "is_integer is_numeric" (this none this) owner: #36 flags: "rxd"
    "Usage:  is_numeric(string)";
    "        is_integer(string)";
    "Is string numeric (composed of one or more digits possibly preceded by a minus sign)? This won't catch floating points.";
    "Return true or false";
    return match(args[1], "^ *[-+]?[0-9]+ *$") ? 1 | 0;
  endverb

  verb "ordinal" (this none this) owner: #36 flags: "rxd"
    ":short_ordinal(1) => \"1st\",:short_ordinal(2) => \"2nd\",etc...";
    string = tostr(n = args[1]);
    n = abs(n) % 100;
    if (n / 10 != 1 && n % 10 in {1, 2, 3})
      return string + {"st", "nd", "rd"}[n % 10];
    else
      return string + "th";
    endif
  endverb

  verb "group_number" (this none this) owner: #36 flags: "rxd"
    "$string_utils:group_number(INT n [, sep_char])";
    "$string_utils:group_number(FLOAT n, [INT precision [, scientific [, sep_char]]])";
    "";
    "Converts N to a string, inserting commas (or copies of SEP_CHAR, if given) every three digits, counting from the right.  For example, $string_utils:group_number(1234567890) returns the string \"1,234,567,890\".";
    "For floats, the arguements precision (defaulting to 4 in this verb) and scientific are the same as given in floatstr().";
    if (typeof(args[1]) == INT)
      {n, ?comma = ","} = args;
      result = "";
      sign = n < 0 ? "-" | "";
      n = tostr(abs(n));
    elseif (typeof(args[1]) == FLOAT)
      {n, ?prec = 4, ?scien = 0, ?comma = ","} = args;
      sign = n < 0.0 ? "-" | "";
      n = floatstr(abs(n), prec, scien);
      i = index(n, ".");
      result = n[i..$];
      n = n[1..i - 1];
    else
      return E_INVARG;
    endif
    while ((len = length(n)) > 3)
      result = comma + n[len - 2..len] + result;
      n = n[1..len - 3];
    endwhile
    return sign + n + result;
    "Code contributed by SunRay";
  endverb

  verb "english_number" (this none this) owner: #36 flags: "rxd"
    "$string_utils:english_number(n) -- convert the integer N into English";
    "";
    "Produces a string containing the English phrase naming the given integer.  For example, $string_utils:english_number(-1234) returns the string `negative one thousand two hundred thirty-four'.";
    numb = toint(args[1]);
    if (numb == 0)
      return "zero";
    endif
    labels = {"", " thousand", " million", " billion"};
    numstr = "";
    mod = abs(numb);
    for n in [1..4]
      div = mod % 1000;
      if (div)
        hun = div / 100;
        ten = div % 100;
        outstr = this:english_tens(ten) + labels[n];
        if (hun)
          outstr = this:english_ones(hun) + " hundred" + (ten ? " " | "") + outstr;
        endif
        if (numstr)
          numstr = outstr + " " + numstr;
        else
          numstr = outstr;
        endif
      endif
      mod = mod / 1000;
    endfor
    return (numb < 0 ? "negative " | "") + numstr;
  endverb

  verb "english_ordinal" (this none this) owner: #36 flags: "rxd"
    "$string_utils:english_ordinal(n) -- convert the integer N into an english ordinal (1 => \"first\", etc...)";
    numb = toint(args[1]);
    if (numb == 0)
      return "zeroth";
    elseif (numb % 100)
      hundreds = abs(numb) > 100 ? this:english_number(numb / 100 * 100) + " " | (numb < 0 ? "negative " | "");
      numb = abs(numb) % 100;
      specials = {1, 2, 3, 5, 8, 9, 12, 20, 30, 40, 50, 60, 70, 80, 90};
      ordinals = {"first", "second", "third", "fifth", "eighth", "ninth", "twelfth", "twentieth", "thirtieth", "fortieth", "fiftieth", "sixtieth", "seventieth", "eightieth", "ninetieth"};
      if (i = numb in specials)
        return hundreds + ordinals[i];
      elseif (numb > 20 && (i = numb % 10 in specials))
        return hundreds + this:english_tens(numb / 10 * 10) + "-" + ordinals[i];
      else
        return hundreds + this:english_number(numb) + "th";
      endif
    else
      return this:english_number(numb) + "th";
    endif
  endverb

  verb "english_ones" (this none this) owner: #36 flags: "rxd"
    numb = args[1];
    ones = {"", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"};
    return ones[numb + 1];
  endverb

  verb "english_tens" (this none this) owner: #36 flags: "rxd"
    numb = args[1];
    teens = {"ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen"};
    others = {"twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"};
    if (numb < 10)
      return this:english_ones(numb);
    elseif (numb < 20)
      return teens[numb - 9];
    else
      return others[numb / 10 - 1] + (numb % 10 ? "-" | "") + this:english_ones(numb % 10);
    endif
  endverb

  verb "subst*itute" (this none this) owner: #36 flags: "rxd"
    "subst(string,{{redex1,repl1},{redex2,repl2},{redex3,repl3}...}[,case])";
    "  => returns string with all instances of the strings redex<n> replaced respectively by the strings repl<n>.  If the optional argument `case' is given and nonzero, the search for instances of redex<n> is case sensitive.";
    "  Substitutions are done in parallel, i.e., instances of redex<n> that appear in any of the replacement strings are ignored.  In the event that two redexes overlap, whichever is leftmost in `string' takes precedence.  For two redexes beginning at the same position, the longer one takes precedence.";
    "";
    "subst(\"hoahooaho\",{{\"ho\",\"XhooX\"},{\"hoo\",\"mama\"}}) => \"XhooXamamaaXhooX\"";
    "subst(\"Cc: banana\",{{\"a\",\"b\"},{\"b\",\"c\"},{\"c\",\"a\"}},1) => \"Ca: cbnbnb\"";
    {input, subs, ?case = 0} = args;
    lines = typeof(input) != LIST ? {input} | input;
    for idx in [1..length(lines)]
      ostr = lines[idx];
      len = length(ostr);
      " - - - find the first instance of each substitution - -";
      indices = {};
      substs = {};
      for s in (subs)
        if (s[1] && (i = index(ostr, s[1], case)))
          fi = $list_utils:find_insert(indices, i = i - len) - 1;
          while (fi && (indices[fi] == i && length(substs[fi][1]) < length(s[1])))
            "...give preference to longer redexes...";
            fi = fi - 1;
          endwhile
          indices = listappend(indices, i, fi);
          substs = listappend(substs, s, fi);
        endif
      endfor
      "- - - - - perform substitutions - ";
      nstr = "";
      while (substs)
        ind = len + indices[1];
        sub = substs[1];
        indices = listdelete(indices, 1);
        substs = listdelete(substs, 1);
        if (ind > 0)
          nstr = nstr + ostr[1..ind - 1] + sub[2];
          ostr = ostr[ind + length(sub[1])..len];
          len = length(ostr);
        endif
        if (next = index(ostr, sub[1], case))
          fi = $list_utils:find_insert(indices, next = next - len) - 1;
          while (fi && (indices[fi] == next && length(substs[fi][1]) < length(sub[1])))
            "...give preference to longer redexes...";
            fi = fi - 1;
          endwhile
          indices = listappend(indices, next, fi);
          substs = listappend(substs, sub, fi);
        endif
      endwhile
      lines[idx] = nstr + ostr;
    endfor
    return typeof(input) == STR ? lines[1] | lines;
  endverb

  verb "substitute_d*elimited" (none none none) owner: #2 flags: "rxd"
    "subst(string,{{redex1,repl1},{redex2,repl2},{redex3,repl3}...}[,case])";
    "Just like :substitute() but it uses index_delimited() instead of index()";
    {ostr, subs, ?case = 0} = args;
    if (typeof(ostr) != STR)
      return ostr;
    endif
    len = length(ostr);
    " - - - find the first instance of each substitution - -";
    indices = {};
    substs = {};
    for s in (subs)
      if (i = this:index_delimited(ostr, s[1], case))
        fi = $list_utils:find_insert(indices, i = i - len) - 1;
        while (fi && (indices[fi] == i && length(substs[fi][1]) < length(s[1])))
          "...give preference to longer redexes...";
          fi = fi - 1;
        endwhile
        indices = listappend(indices, i, fi);
        substs = listappend(substs, s, fi);
      endif
    endfor
    "- - - - - perform substitutions - ";
    nstr = "";
    while (substs)
      ind = len + indices[1];
      sub = substs[1];
      indices = listdelete(indices, 1);
      substs = listdelete(substs, 1);
      if (ind > 0)
        nstr = nstr + ostr[1..ind - 1] + sub[2];
        ostr = ostr[ind + length(sub[1])..len];
        len = length(ostr);
      endif
      if (next = this:index_delimited(ostr, sub[1], case))
        fi = $list_utils:find_insert(indices, next = next - len) - 1;
        while (fi && (indices[fi] == next && length(substs[fi][1]) < length(sub[1])))
          "...give preference to longer redexes...";
          fi = fi - 1;
        endwhile
        indices = listappend(indices, next, fi);
        substs = listappend(substs, sub, fi);
      endif
    endwhile
    return nstr + ostr;
  endverb

  verb "_cap_property" (this none this) owner: #36 flags: "rxd"
    "cap_property(what,prop [,ucase [,title]]) returns what.(prop) but capitalized if either ucase is true or the prop name specified is capitalized.";
    "If prop is blank, returns what:title().";
    "If prop is bogus or otherwise irretrievable, returns the error.";
    "If capitalization is indicated, we return what.(prop+\"c\") if that exists, else we capitalize what.(prop) in the usual fashion.  There is a special exception for is_player(what)&&prop==\"name\" where we just return what.name if no .namec is provided --- ie., a player's .name is never capitalized in the usual fashion.";
    "If args[1] is a list, calls itself on each element of the list and returns $string_utils:english_list(those results).";
    {what, prop, ?ucase = 0, ?titlemode = 0} = args;
    set_task_perms(caller_perms());
    if (typeof(what) == LIST)
      result = {};
      for who in (what)
        result = {@result, this:_cap_property(who, prop, ucase, titlemode)};
      endfor
      return $string_utils:english_list(result);
    endif
    ucase = prop && strcmp(prop, "a") < 0 || ucase;
    name = `titlemode ? what:title() | what:name() ! E_INVIND => "something"';
    if (!prop)
      temp = valid(what) ? ucase ? $su:capitalize(name) | name | (ucase ? "N" | "n") + "othing";
      return temp;
    elseif (!ucase || typeof(s = `what.(prop + "c") ! ANY') == ERR)
      if (prop == "name")
        s = valid(what) ? what:name() | "nothing";
        ucase = ucase && !is_player(what);
      else
        s = `$object_utils:has_property(what, prop) ? what.(prop) | $player.(prop) ! ANY';
      endif
      if (ucase && (s && (typeof(s) == STR && ((z = index(this.alphabet, s[1], 1)) < 27 && z > 0))))
        s[1] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"[z];
      endif
    endif
    return typeof(s) == ERR ? s | tostr(s);
  endverb

  verb "pronoun_sub" (this none this) owner: #36 flags: "rxd"
    "Pronoun (and other things) substitution. See 'help pronouns' for details.";
    "syntax:  $string_utils:pronoun_sub(text[,who[,thing[,location[,dobj[,iobj[,phrase]]]]]])";
    "%s,%o,%p,%q,%r       => <who>'s pronouns.  <who> defaults to player.";
    "%f,%g,%h             => <who>'s SIC fullname, first name, or alias";
    "%n,%d,%i,%t,%l,%a,%% => <who>, dobj, iobj, <thing>, location , phrase and %";
    "<thing> defaults to caller; <location> defaults to who.location";
    "<phrase> is can be any string value";
    "%S,%O,%P,%Q,%R, %N,%D,%I,%T,%L have corresponding capitalized substitutions.";
    " %[#n], %[#d], ...  =>  <who>, dobj, etc.'s object number";
    "%(foo) => <who>.foo and %(Foo) => <who>.foo capitalized. %[dfoo] => dobj.foo, etc..";
    "%<foo> -> whatever <who> does when normal people foo. This is determined by calling :verb_sub() on the <who>.";
    "%<d:foo> -> whatever <dobj> does when normal people foo.";
    {string, ?who = player, ?thing = caller, ?where = $nothing, @more_args} = args;
    {?phrase = "UNDEF", ?titlemode = 0, @even_more_args} = more_args;
    phrase_too = "UNDEF";
    if (typeof(phrase) == LIST)
      phrase_too = length(phrase) > 1 ? phrase[2] | phrase_too;
      phrase = `phrase[1] ! E_RANGE => "UNDEF"';
    endif
    where = valid(where) ? where | (valid(who) ? who.location | where);
    set_task_perms($no_one);
    if (typeof(string) == LIST)
      plines = {};
      for line in (string)
        
        plines = {@plines, this:(verb)(line, @args[2..$])};
      endfor
      return plines;
    endif
    old = tostr(string);
    new = "";
    objspec = "nditlabfgh";
    objects = {who, dobj, iobj, thing, where, phrase, phrase_too, "f", "g", "h"};
    prnspec = "sopqrSOPQR";
    prprops = {"ps", "po", "pp", "pq", "pr", "Ps", "Po", "Pp", "Pq", "Pr"};
    oldlen = length(old);
    while ((prcnt = index(old, "%")) && prcnt < oldlen)
      s = old[k = prcnt + 1];
      cp_args = {};
      if (brace = index("([", s))
        if (!(w = index(old[k + 1..oldlen], ")]"[brace])))
          return new + old;
        else
          p = old[prcnt + 2..(k = k + w) - 1];
          if (brace == 1)
            "%(property)";
            cp_args = {who, p};
          elseif (p[1] == "#")
            "%[#n] => object number";
            s = (o = index(objspec, p[2])) ? tostr(objects[o]) | "[" + p + "]";
          elseif (!(o = index(objspec, p[1])))
            s = "[" + p + "]";
          else
            " %[dproperty] ";
            cp_args = {objects[o], p[2..w - 1], strcmp(p[1], "a") < 0};
          endif
        endif
      elseif (o = index(objspec, s))
        cp_args = {objects[o], "", strcmp(s, "a") < 0};
      elseif (w = index(prnspec, s, 1))
        cp_args = {who, prprops[w], 0};
        if (s == "S" && $gender_utils.is_plural[`who.gender ! ANY => "neuter"' in $gender_utils.genders])
          "read ahead replace";
          nws = prcnt + 3;
          nwe = prcnt + 1 + index(old[prcnt + 3..$], " ");
          new_word = old[nws..nwe];
          if (new_word == "is")
            old[nws..nwe] = "are";
          elseif ($su:is_plural(new_word))
            old[nws..nwe] = $su:singularize(new_word);
          endif
        endif
      elseif (s == "#")
        s = tostr(who);
      elseif (s != "%")
        s = "%" + s;
      endif
      val = "";
      if (!cp_args)
        val = s;
      elseif (s == "a")
        val = phrase;
      elseif (s == "b")
        val = phrase_too;
      elseif (s == "f")
        val = `who:sic_fullname() ! E_VERBNF => "<Error>"';
      elseif (s == "g")
        val = `who:sic_name() ! E_VERBNF => "<Error>"';
      elseif (s == "h")
        val = `$su:trim(who:sic_alias()) ! E_VERBNF => "<Error>"';
      elseif (typeof(sub = $su:_cap_property(@cp_args, titlemode)) == ERR)
        val = "%(" + tostr(sub) + ")";
      else
        val = sub;
      endif
      new = new + old[1..prcnt - 1] + val;
      old = old[k + 1..$];
      oldlen = oldlen - k;
    endwhile
    return new + old;
  endverb

  verb "pronoun_sub_secure" (this none this) owner: #36 flags: "rxd"
    "$string_utils:pronoun_sub_secure(string[,who[,thing[,location]]], default)";
    "Do pronoun_sub on string with the arguments given (see help";
    "string_utils:pronoun_sub for more information).  Return pronoun_subbed";
    "<default> if the subbed string does not contain <who>.name (<who>";
    "defaults to player).";
    who = length(args) > 2 ? args[2] | player;
    default = args[$];
    result = this:pronoun_sub(@args[1..$ - 1]);
    return this:index_delimited(result, who.name) ? result | this:pronoun_sub(@{default, @args[2..$ - 1]});
  endverb

  verb "pronoun_quote" (this none this) owner: #36 flags: "rxd"
    " pronoun_quote(string) => quoted_string";
    " pronoun_quote(list of strings) => list of quoted_strings";
    " pronoun_quote(list of {key,string} pairs) => list of {key,quoted_string} pairs";
    "";
    "Here `quoted' means quoted in the sense of $string_utils:pronoun_sub, i.e., given a string X, the corresponding `quoted' string Y is such that pronoun_sub(Y) => X.  For example, pronoun_quote(\"--%Spam%--\") => \"--%%Spam%%--\".  This is for including literal text into a string that will eventually be pronoun_sub'ed, i.e., including it in such a way that the pronoun_sub will not expand anything in the included text.";
    "";
    "The 3rd form above (with {key,string} pairs) is for use with $string_utils:substitute().  If you have your own set of substitutions to be done in parallel with the pronoun substitutions, do";
    "";
    "  msg=$string_utils:substitute(msg,$string_utils:pronoun_quote(your_substs));";
    "  msg=$string_utils:pronoun_sub(msg);";
    if (typeof(what = args[1]) == STR)
      return strsub(what, "%", "%%");
    else
      ret = {};
      for w in (what)
        if (typeof(w) == LIST)
          ret = listappend(ret, listset(w, strsub(w[2], "%", "%%"), 2));
        else
          ret = listappend(ret, strsub(w, "%", "%%"));
        endif
      endfor
      return ret;
    endif
  endverb

  verb "alt_pronoun_sub" (none none none) owner: #2 flags: "rxd"
    "Pronoun (and other things) substitution. See 'help pronouns' for details.";
    "syntax:  $string_utils:pronoun_sub(text[,who[,thing[,location]]])";
    "%s,%o,%p,%q,%r    => <who>'s pronouns.  <who> defaults to player.";
    "%n,%d,%i,%t,%l,%% => <who>, dobj, iobj, this, <who>.location and %";
    "%S,%O,%P,%Q,%R, %N,%D,%I,%T,%L have corresponding capitalized substitutions.";
    " %[#n], %[#d], ...  =>  <who>, dobj, etc.'s object number";
    "%(foo) => <who>.foo and %(Foo) => <who>.foo capitalized. %[dfoo] => dobj.foo, etc..";
    "%<foo> -> whatever <who> does when normal people foo. This is determined by calling :verb_sub() on the <who>.";
    "%<d:foo> -> whatever <dobj> does when normal people foo.";
    set_task_perms($no_one);
    {string, ?who = player, ?thing = caller, ?where = $nothing} = args;
    where = valid(who) ? who.location | where;
    if (typeof(string) == LIST)
      plines = {};
      for line in (string)
        plines = {@plines, this:(verb)(line, who, thing, where)};
      endfor
      return plines;
    endif
    old = tostr(string);
    new = "";
    objspec = "nditl";
    objects = {who, dobj, iobj, thing, where};
    prnspec = "sopqrSOPQR";
    prprops = {"ps", "po", "pp", "pq", "pr", "Ps", "Po", "Pp", "Pq", "Pr"};
    oldlen = length(old);
    while ((prcnt = index(old, "%")) && prcnt < oldlen)
      s = old[k = prcnt + 1];
      if (s == "<" && (gt = index(old[k + 2..$], ">")))
        "handling %<verb> ";
        gt = gt + k + 1;
        vb = old[k + 1..gt - 1];
        vbs = who;
        if (length(vb) > 2 && vb[2] == ":")
          " %<d:verb>";
          vbs = objects[index(objspec, vb[1]) || 1];
          vb = vb[3..$];
        endif
        vb = $object_utils:has_verb(vbs, "verb_sub") ? vbs:verb_sub(vb) | this:(verb)(vb, vbs);
        new = new + old[1..prcnt - 1] + vb;
        k = gt;
      else
        cp_args = {};
        if (brace = index("([", s))
          if (!(w = index(old[k + 1..oldlen], ")]"[brace])))
            return new + old;
          else
            p = old[prcnt + 2..(k = k + w) - 1];
            if (brace == 1)
              "%(property)";
              cp_args = {who, p};
            elseif (p[1] == "#")
              "%[#n] => object number";
              s = (o = index(objspec, p[2])) ? tostr(objects[o]) | "[" + p + "]";
            elseif (!(o = index(objspec, p[1])))
              s = "[" + p + "]";
            else
              " %[dproperty] ";
              cp_args = {objects[o], p[2..w - 1], strcmp(p[1], "a") < 0};
            endif
          endif
        elseif (o = index(objspec, s))
          cp_args = {objects[o], "", strcmp(s, "a") < 0};
        elseif (w = index(prnspec, s, 1))
          cp_args = {who, prprops[w]};
        elseif (s == "#")
          s = tostr(who);
        elseif (s != "%")
          s = "%" + s;
        endif
        new = new + old[1..prcnt - 1] + (!cp_args ? s | (typeof(sub = $string_utils:_cap_property(@cp_args)) != ERR ? sub | "%(" + tostr(sub) + ")"));
      endif
      old = old[k + 1..oldlen];
      oldlen = oldlen - k;
    endwhile
    return new + old;
  endverb

  verb "explode split" (this none this) owner: #36 flags: "rxd"
    "$string_utils:explode(subject [, break [, full]])";
    "Return a list of those substrings of subject separated by runs of break[1].";
    "If 'full' is provided and true, the break is a full string";
    "break defaults to space.";
    "THIS VERSION DOES NOT SUPPORT empty strings between break markers";
    "if you need empty string support, use $su:explode_all";
    {subject, ?breakit = {" "}, ?full = 0} = args;
    if (!full)
      breakit = breakit[1];
    endif
    subject = subject + breakit;
    parts = {};
    while (subject)
      if ((i = index(subject, breakit)) > 1)
        parts = {@parts, subject[1..i - 1]};
      endif
      subject = subject[i + length(breakit)..$];
    endwhile
    return parts;
  endverb

  verb "words" (this none this) owner: #36 flags: "rxd"
    "This breaks up the argument string into words, the resulting list being obtained exactly the way the command line parser obtains `args' from `argstr'.";
    rest = args[1];
    "...trim leading blanks...";
    if (0)
      rest[1..match(rest, "^ *")[2]] = "";
    endif
    rest = $string_utils:triml(rest);
    if (!rest)
      return {};
    endif
    quote = 0;
    toklist = {};
    token = "";
    pattern = " +%|\\.?%|\"";
    while (m = match(rest, pattern))
      "... find the next occurence of a special character, either";
      "... a block of spaces, a quote or a backslash escape sequence...";
      char = rest[m[1]];
      token = token + rest[1..m[1] - 1];
      if (char == " ")
        toklist = {@toklist, token};
        token = "";
      elseif (char == "\"")
        "... beginning or end of quoted string...";
        "... within a quoted string spaces aren't special...";
        pattern = (quote = !quote) ? "\\.?%|\"" | " +%|\\.?%|\"";
      elseif (m[1] < m[2])
        "... char has to be a backslash...";
        "... include next char literally if there is one";
        token = token + rest[m[2]];
      endif
      rest[1..m[2]] = "";
    endwhile
    return rest || char != " " ? {@toklist, token + rest} | toklist;
  endverb

  verb "word_start" (this none this) owner: #36 flags: "rxd"
    "This breaks up the argument string into words, returning a list of indices into argstr corresponding to the starting points of each of the arguments.";
    rest = args[1];
    "... find first nonspace...";
    wstart = match(rest, "[^ ]%|$")[1];
    wbefore = wstart - 1;
    rest[1..wbefore] = "";
    if (!rest)
      return {};
    endif
    quote = 0;
    wslist = {};
    pattern = " +%|\\.?%|\"";
    while (m = match(rest, pattern))
      "... find the next occurence of a special character, either";
      "... a block of spaces, a quote or a backslash escape sequence...";
      char = rest[m[1]];
      if (char == " ")
        wslist = {@wslist, {wstart, wbefore + m[1] - 1}};
        wstart = wbefore + m[2] + 1;
      elseif (char == "\"")
        "... beginning or end of quoted string...";
        "... within a quoted string spaces aren't special...";
        pattern = (quote = !quote) ? "\\.?%|\"" | " +%|\\.?%|\"";
      endif
      rest[1..m[2]] = "";
      wbefore = wbefore + m[2];
    endwhile
    return rest || char != " " ? {@wslist, {wstart, wbefore + length(rest)}} | wslist;
  endverb

  verb "to_value" (this none this) owner: #36 flags: "rxd"
    ":to_value(string) tries to parse string as a value (i.e., object, number, string, error, or list thereof).";
    "Returns {1,value} or {0,error_message} according as the attempt was successful or not.";
    string = this:triml(args[1]);
    if (string[1] == "[" || string[$] == "]")
      result = this:_tomap(string[1] == "[" ? string[2..$] | string);
      if (typeof(result[2]) != MAP)
        return {0, result[2]};
      else
        return {1, result[2]};
      endif
    else
      result = this:_tolist(string = args[1] + "}");
      if (result[1] && result[1] != $string_utils:space(result[1]))
        return {0, tostr("after char ", length(string) - result[1], ":  ", result[2])};
      elseif (typeof(result[1]) == INT)
        return {0, "missing } or \""};
      elseif (length(result[2]) > 1)
        return {0, "comma unexpected."};
      elseif (result[2])
        return {1, typeof(result[2]) == LIST ? result[2][1] | result[2]};
      else
        return {0, "missing expression"};
      endif
    endif
  endverb

  verb "prefix_to_value" (this none this) owner: #36 flags: "rxd"
    ":prefix_to_value(string) tries to parse string as a value (i.e., object, number, string, error, or list thereof).";
    "Returns {rest-of-string,value} or {0,error_message} according as the attempt was successful or not.";
    alen = length(args[1]);
    slen = length(string = this:triml(args[1]));
    if (!string)
      return {0, "empty string"};
    elseif (w = index("{[\"", string[1]))
      result = this:({"_tolist", "_tomap", "_unquote"}[w])(string[2..slen]);
      if (typeof(result[1]) != INT)
        return result;
      elseif (result[1] == 0)
        return {0, "missing } or \""};
      else
        return {0, result[2], alen - result[1] + 1};
      endif
    else
      thing = string[1..tlen = index(string + " ", " ") - 1];
      if (typeof(s = this:_toscalar(thing)) != STR)
        return {string[tlen + 1..slen], s};
      else
        return {0, s, alen - slen + 1};
      endif
    endif
  endverb

  verb "_tolist" (this none this) owner: #36 flags: "rxd"
    "_tolist(string) --- auxiliary for :to_value()";
    rest = this:triml(args[1]);
    vlist = {};
    if (!rest)
      return {0, {}};
    elseif (rest[1] == "}")
      return {rest[2..$], {}};
    endif
    while (1)
      rlen = length(rest);
      if (w = index("{\"", rest[1]))
        result = this:({"_tolist", "_unquote"}[w])(rest[2..rlen]);
        if (typeof(result[1]) == INT)
          return result;
        endif
        vlist = {@vlist, result[2]};
        rest = result[1];
      else
        thing = rest[1..tlen = min(index(rest + ",", ","), index(rest + "}", "}")) - 1];
        if (typeof(s = this:_toscalar(thing)) == STR)
          return {rlen, s};
        endif
        vlist = {@vlist, s};
        rest = rest[tlen + 1..rlen];
      endif
      if (!rest)
        return {0, vlist};
      elseif (rest[1] == "}")
        return {rest[2..$], vlist};
      elseif (rest[1] == ",")
        rest = this:triml(rest[2..$]);
      else
        return {length(rest), ", or } expected"};
      endif
    endwhile
  endverb

  verb "_unquote" (this none this) owner: #36 flags: "rxd"
    "_unquote(string)   (auxiliary for :to_value())";
    "reads string as if it were preceded by a quote, reading up to the closing quote if any, then returns the corresponding unquoted string.";
    " => {0, string unquoted}  if there is no closing quote";
    " => {original string beyond closing quote, string unquoted}  otherwise";
    rest = args[1];
    result = "";
    while (m = match(rest, "\\.?%|\""))
      "Find the next special character";
      if (rest[pos = m[1]] == "\"")
        return {rest[pos + 1..$], result + rest[1..pos - 1]};
      endif
      result = result + rest[1..pos - 1] + rest[pos + 1..m[2]];
      rest = rest[m[2] + 1..$];
    endwhile
    return {0, result + rest};
  endverb

  verb "_toscalar" (this none this) owner: #36 flags: "rxd"
    ":_toscalar(string)  --- auxiliary for :tovalue";
    " => value if string represents a number, object or error";
    " => string error message otherwise";
    thing = args[1];
    if (!thing)
      return "missing value";
    elseif (match(thing, "^#?[-+]?[0-9]+ *$"))
      return thing[1] == "#" ? toobj(thing) | toint(thing);
    elseif (match(thing, "^[-+]?%([0-9]+%.[0-9]*%|[0-9]*%.[0-9]+%)%(e[-+]?[0-9]+%)? *$"))
      "matches 2. .2 3.2 3.2e3 .2e-3 3.e3";
      return `tofloat(thing) ! E_INVARG => tostr("Bad floating point value: ", thing)';
    elseif (match(thing, "^[-+]?[0-9]+e[-+]?[0-9]+ *$"))
      "matches 345e4. No decimal, but has an e so still a float";
      return `tofloat(thing) ! E_INVARG => tostr("Bad floating point value: ", thing)';
    elseif (thing == "true")
      return true;
    elseif (thing == "false")
      return false;
    elseif (thing[1] == "E")
      return (e = $code_utils:toerr(thing)) ? tostr("unknown error code `", thing, "'") | e;
    elseif (thing[1] == "#")
      return tostr("bogus objectid `", thing, "'");
    else
      return tostr("`", thing[1], "' unexpected");
    endif
  endverb

  verb "parse_command" (this none this) owner: #2 flags: "rxd"
    ":parse_command(cmd_line[,player])";
    " => {verb, {dobj, dobjstr}, {prep, prepstr}, {iobj, iobjstr}, {args, argstr},";
    "     {dobjset, prepset, iobjset}}";
    "This mimics the action of the builtin parser, returning what the values of the builtin variables `verb', `dobj', `dobjstr', `prepstr', `iobj', `iobjstr', `args', and `argstr' would be if `player' had typed `cmd_line'.  ";
    "`prep' is the shortened version of the preposition found.";
    "";
    "`dobjset' and `iobjset' are subsets of {\"any\",\"none\"} and are used to determine possible matching verbs, i.e., the matching verb must either be on `dobj' and have verb_args[1]==\"this\" or else it has verb_args[1] in `dobjset'; likewise for `iobjset' and verb_args[3]; similarly we must have verb_args[2] in `prepset'.";
    {c, ?who = player} = args;
    y = $string_utils:words(c);
    if (y == {})
      return {};
    endif
    vrb = y[1];
    y = y[2..$];
    as = y == {} ? "" | c[length(vrb) + 2..$];
    n = 1;
    while (!(gp = $code_utils:get_prep(@y[n..$]))[1] && n < length(y))
      n = n + 1;
    endwhile
    "....";
    really = player;
    player = who;
    loc = who.location;
    if (ps = gp[1])
      ds = $string_utils:from_list(y[1..n - 1], " ");
      is = $string_utils:from_list(listdelete(gp, 1), " ");
      io = valid(loc) ? loc:match_object(is) | $string_utils:match_object(is, loc);
    else
      ds = $string_utils:from_list(y, " ");
      is = "";
      io = $nothing;
    endif
    do = valid(loc) ? loc:match_object(ds) | $string_utils:match_object(ds, loc);
    player = really;
    "....";
    dset = {"any", @ds == "" ? {"none"} | {}};
    "\"this\" must be handled manually.";
    pset = {"any", @ps ? {$code_utils:full_prep(ps)} | {"none"}};
    iset = {"any", @is == "" ? {"none"} | {}};
    return {vrb, {do, ds}, {$code_utils:short_prep(ps), ps}, {io, is}, {y, as}, {dset, pset, iset}};
  endverb

  verb "from_value" (this none this) owner: #2 flags: "rxd"
    "$string_utils:from_value(value [, quote_strings = 0 [, list_depth = 1]])";
    "Print the given value into a string.";
    {value, ?quote_strings = 0, ?list_depth = 1} = args;
    if (typeof(value) == LIST)
      if (value)
        if (list_depth)
          result = "{" + this:from_value(value[1], quote_strings, list_depth - 1);
          for v in (listdelete(value, 1))
            result = tostr(result, ", ", this:from_value(v, quote_strings, list_depth - 1));
          endfor
          return result + "}";
        else
          return "{...}";
        endif
      else
        return "{}";
      endif
    elseif (quote_strings)
      if (typeof(value) == STR)
        result = "\"";
        while (q = index(value, "\"") || index(value, "\\"))
          if (value[q] == "\"")
            q = min(q, index(value + "\\", "\\"));
          endif
          result = result + value[1..q - 1] + "\\" + value[q];
          value = value[q + 1..$];
        endwhile
        return result + value + "\"";
      elseif (typeof(value) == ERR)
        return $code_utils:error_name(value);
      else
        return tostr(value);
      endif
    else
      return tostr(value);
    endif
  endverb

  verb "print" (this none this) owner: #36 flags: "rxd"
    "$string_utils:print(value)";
    "Print the given value into a string. == from_value(value,1,-1)";
    return toliteral(args[1]);
  endverb

  verb "reverse" (this none this) owner: #36 flags: "rxd"
    ":reverse(string) => \"gnirts\"";
    "An example: :reverse(\"This is a test.\") => \".tset a si sihT\"";
    string = args[1];
    if ((len = length(string)) > 50)
      return this:reverse(string[$ / 2 + 1..$]) + this:reverse(string[1..$ / 2]);
    endif
    index = len;
    result = "";
    while (index > 0)
      result = result + string[index];
      index = index - 1;
    endwhile
    return result;
  endverb

  verb "char_list" (this none this) owner: #36 flags: "rxd"
    ":char_list(string) => string as a list of characters.";
    "   e.g., :char_list(\"abad\") => {\"a\",\"b\",\"a\",\"d\"}";
    if (30 < (len = length(string = args[1])))
      return {@this:char_list(string[1..$ / 2]), @this:char_list(string[$ / 2 + 1..$])};
    else
      l = {};
      for c in [1..len]
        l = {@l, string[c]};
      endfor
      return l;
    endif
  endverb

  verb "regexp_quote" (this none this) owner: #36 flags: "rxd"
    ":regexp_quote(string)";
    " => string with all of the regular expression special characters quoted with %";
    string = args[1];
    quoted = "";
    while (m = rmatch(string, "[][$^.*+?%].*"))
      quoted = "%" + string[m[1]..m[2]] + quoted;
      string = string[1..m[1] - 1];
    endwhile
    return string + quoted;
  endverb

  verb "connection_hostname" (this none this) owner: #2 flags: "rxd"
    "Return the host string for an object or extract it from a legacy connection_name() string. Assumes you are using bsd_network style connection names.";
    caller != #0 && set_task_perms(caller_perms());
    {lookup} = args;
    if (typeof(lookup) == OBJ)
      return `connection_name(lookup) ! E_INVARG => ""';
    elseif (typeof(lookup) == STR)
      "Make the assumption here that connection_name() has been passed in from legacy code and contains just the host string.";
      return (m = `match(lookup, "^.* %(from%|to%) %([^, ]+%)") ! ANY') ? substitute("%2", m) | lookup;
    else
      return "";
    endif
  endverb

  verb "end_expression" (this none this) owner: #36 flags: "rxd"
    ":end_expression(string[,stop_at])";
    "  assumes string starts with an expression; returns the index of the last char in expression or 0 if string appears not to be an expression.  Expression ends at any character from stop_at which occurs at top level.";
    {string, ?stop_at = " "} = args;
    gone = 0;
    paren_stack = "";
    inquote = 0;
    search = top_level_search = "[][{}()\"" + strsub(stop_at, "]", "") + "]";
    paren_search = "[][{}()\"]";
    while (m = match(string, search))
      char = string[m[1]];
      string[1..m[2]] = "";
      gone = gone + m[2];
      if (char == "\"")
        "...skip over quoted string...";
        char = "\\";
        while (char == "\\")
          if (!(m = match(string, "%(\\.?%|\"%)")))
            return 0;
          endif
          char = string[m[1]];
          string[1..m[2]] = "";
          gone = gone + m[2];
        endwhile
      elseif (index("([{", char))
        "... push parenthesis...";
        paren_stack[1..0] = char;
        search = paren_search;
      elseif (i = index(")]}", char))
        if (paren_stack && "([{"[i] == paren_stack[1])
          "... pop parenthesis...";
          paren_stack[1..1] = "";
          search = paren_stack ? paren_search | top_level_search;
        else
          "...parenthesis mismatch...";
          return 0;
        endif
      else
        "... stop character ...";
        return gone - 1;
      endif
    endwhile
    return !paren_stack && gone + length(string);
  endverb

  verb "first_word" (this none this) owner: #36 flags: "rxd"
    ":first_word(string) => {first word, rest of string} or {}";
    rest = args[1];
    "...trim leading blanks...";
    rest[1..match(rest, "^ *")[2]] = "";
    if (!rest)
      return {};
    endif
    quote = 0;
    token = "";
    pattern = " +%|\\.?%|\"";
    while (m = match(rest, pattern))
      "... find the next occurence of a special character, either";
      "... a block of spaces, a quote or a backslash escape sequence...";
      char = rest[m[1]];
      token = token + rest[1..m[1] - 1];
      if (char == " ")
        rest[1..m[2]] = "";
        return {token, rest};
      elseif (char == "\"")
        "... beginning or end of quoted string...";
        "... within a quoted string spaces aren't special...";
        pattern = (quote = !quote) ? "\\.?%|\"" | " +%|\\.?%|\"";
      elseif (m[1] < m[2])
        "... char has to be a backslash...";
        "... include next char literally if there is one";
        token = token + rest[m[2]];
      endif
      rest[1..m[2]] = "";
    endwhile
    return {token + rest, ""};
  endverb

  verb "common" (this none this) owner: #36 flags: "rxd"
    ":common(first,second) => length of longest common prefix";
    {first, second} = args;
    r = min(length(first), length(second));
    l = 1;
    while (r >= l)
      h = (r + l) / 2;
      if (first[l..h] == second[l..h])
        l = h + 1;
      else
        r = h - 1;
      endif
    endwhile
    return r;
  endverb

  verb "title_list*c list_title*c" (this none this) owner: #36 flags: "rxd"
    "wr_utils:title_list/title_listc(<obj-list>[, @<args>)";
    "Creates an english list out of the titles of the objects in <obj-list>.  Optional <args> are passed on to $string_utils:english_list.";
    "title_listc uses :titlec() for the first item.";
    titles = $list_utils:map_verb(args[1], "title");
    if (verb[length(verb)] == "c")
      if (titles)
        titles[1] = args[1][1]:titlec();
      elseif (length(args) > 1)
        args[2] = $string_utils:capitalize(args[2]);
      else
        args = listappend(args, "Nothing");
      endif
    endif
    return $string_utils:english_list(titles, @args[2..$]);
  endverb

  verb "name_and_number nn name_and_number_list nn_list" (this none this) owner: #36 flags: "rxd"
    "name_and_number(object [,sepr] [,english_list_args]) => \"ObjectName (#object)\"";
    "Return name and number for OBJECT.  Second argument is optional separator (for those who want no space, use \"\").  If OBJECT is a list of objects, this maps the above function over the list and then passes it to $string_utils:english_list.";
    "The third through nth arguments to nn_list corresponds to the second through nth arguments to English_list, and are passed along untouched.";
    {objs, ?sepr = " ", @eng_args} = args;
    if (typeof(objs) != LIST)
      objs = {objs};
    endif
    name_list = {};
    for what in (objs)
      name = valid(what) ? what.name | {"<invalid>", "$nothing", "$ambiguous_match", "$failed_match"}[1 + (what in {#-1, #-2, #-3})];
      name = tostr(name, sepr, "(", what, ")");
      name_list = {@name_list, name};
    endfor
    return this:english_list(name_list, @eng_args);
  endverb

  verb "a_or_an" (this none this) owner: #36 flags: "rxd"
    ":a_or_an(<noun>) => \"a\" or \"an\"";
    "To accomodate personal variation (e.g., \"an historical book\"), a player can override this by having a personal a_or_an verb.  If that verb returns 0 instead of a string, the standard algorithm is used.";
    noun = args[1];
    if ($object_utils:has_verb(player, "a_or_an") && (custom_result = player:a_or_an(noun)) != 0)
      return custom_result;
    endif
    if (noun in this.use_article_a)
      return "a";
    endif
    if (noun in this.use_article_an)
      return "an";
    endif
    a_or_an = "a";
    if (noun != "")
      if (index("aeiou", noun[1]))
        a_or_an = "an";
        "unicycle, unimplemented, union, united, unimpressed, unique";
        if (noun[1] == "u" && length(noun) > 2 && noun[2] == "n" && (index("aeiou", noun[3]) == 0 || (noun[3] == "i" && length(noun) > 3 && (index("aeioubcghqwyz", noun[4]) || (length(noun) > 4 && index("eiy", noun[5]))))))
          a_or_an = "a";
        endif
      endif
    endif
    return a_or_an;
    "Ported by Mickey with minor tweaks from a Moo far far away.";
    "Last modified Sun Aug  1 22:53:07 1993 EDT by BabyBriar (#2).";
  endverb

  verb "index_all" (this none this) owner: #36 flags: "rxd"
    "index_all(string,target) -- returns list of positions of target in string.";
    "Usage: $string_utils:index_all(<string,pattern>)";
    "       $string_utils:index_all(\"aaabacadae\",\"a\")";
    {line, pattern} = args;
    if (typeof(line) != STR || typeof(pattern) != STR)
      return E_TYPE;
    else
      where = {};
      place = -1;
      next = 0;
      while ((place = index(line[next + 1..$], pattern)) != 0)
        where = {@where, place + next};
        next = place + next + length(pattern) - 1;
      endwhile
      return where;
    endif
  endverb

  verb "match_stringlist match_string_list" (this none this) owner: #36 flags: "rx"
    "$string_utils:match_stringlist(string, {list of strings})";
    "  Returns 0 if no match found";
    "  Otherwise returns the index of the FIRST match";
    {subject, strings} = args;
    if (subject == "" || length(strings) < 1)
      return 0;
    endif
    "First check for exact matches.";
    for string, index in (strings)
      if (string == subject)
        return index;
      endif
    endfor
    "now check for partial matches";
    for string, index in (strings)
      if (subject in string)
        return index;
      endif
    endfor
    return 0;
  endverb

  verb "from_ASCII" (this none this) owner: #36 flags: "rxd"
    "This converts a ASCII character code in the range [32..126] into the ASCII character with that code, represented as a one-character string.";
    "";
    "Example:   $string_utils:from_ASCII(65) => \"A\"";
    code = args[1];
    return this.ascii[code - 31];
  endverb

  verb "to_ASCII" (this none this) owner: #36 flags: "rxd"
    "Convert a one-character string into the ASCII character code for that character.";
    "";
    "Example:  $string_utils:to_ASCII(\"A\") => 65";
    return (index(this.ascii, args[1], 1) || raise(E_INVARG)) + 31;
  endverb

  verb "abbreviated_value" (this none this) owner: #36 flags: "rxd"
    "Copied from Mickey (#52413):abbreviated_value Fri Sep  9 08:52:41 1994 PDT";
    ":abbreviated_value(value,max_reslen,max_lstlev,max_lstlen,max_strlen,max_toklen)";
    "";
    "Gets the printed representation of value, subject to these parameters:";
    " max_reslen = Maximum desired result string length.";
    " max_lstlev = Maximum list level to show.";
    " max_lstlen = Maximum list length to show.";
    " max_strlen = Maximum string length to show.";
    " max_toklen = Maximum token length (e.g., numbers and errors) to show.";
    "";
    "A best attempt is made to get the exact target size, but in some cases the result is not exact.";
    {value, ?max_reslen = $maxint, ?max_lstlev = $maxint, ?max_lstlen = $maxint, ?max_strlen = $maxint, ?max_toklen = $maxint} = args;
    return this:_abbreviated_value(value, max_reslen, max_lstlev, max_lstlen, max_strlen, max_toklen);
    "Originally written by Mickey.";
  endverb

  verb "_abbreviated_value" (this none this) owner: #36 flags: "rxd"
    "Copied from Mickey (#52413):_abbreviated_value Fri Sep  9 08:52:44 1994 PDT";
    "Internal to :abbreviated_value.  Do not call this directly.";
    {value, max_reslen, max_lstlev, max_lstlen, max_strlen, max_toklen} = args;
    if ((type = typeof(value)) == LIST)
      if (!value)
        return "{}";
      elseif (max_lstlev == 0)
        return "{...}";
      else
        n = length(value);
        result = "{";
        r = max_reslen - 2;
        i = 1;
        eltstr = "";
        while (i <= n && i <= max_lstlen && r > (x = i == 1 ? 0 | 2))
          eltlen = length(eltstr = this:(verb)(value[i], r, max_lstlev - 1, max_lstlen, max_strlen, max_toklen));
          lastpos = 1;
          if (r >= eltlen + x)
            comma = i == 1 ? "" | ", ";
            result = tostr(result, comma);
            if (r > 4)
              lastpos = length(result);
            endif
            result = tostr(result, eltstr);
            r = r - eltlen - x;
          elseif (i == 1)
            return "{...}";
          elseif (r > 4)
            return tostr(result, ", ...}");
          else
            return tostr(result[1..lastpos], "...}");
          endif
          i = i + 1;
        endwhile
        if (i <= n)
          if (i == 1)
            return "{...}";
          elseif (r > 4)
            return tostr(result, ", ...}");
          else
            return tostr(result[1..lastpos], "...}");
          endif
        else
          return tostr(result, "}");
        endif
      endif
    elseif (type == STR)
      result = "\"";
      while ((q = index(value, "\"")) ? q = min(q, index(value, "\\")) | (q = index(value, "\\")))
        result = result + value[1..q - 1] + "\\" + value[q];
        value = value[q + 1..$];
      endwhile
      result = result + value;
      if (length(result) + 1 > (z = max(min(max_reslen, max(max_strlen, max_strlen + 2)), 6)))
        z = z - 5;
        k = 0;
        while (k < z && result[z - k] == "\\")
          k = k + 1;
        endwhile
        return tostr(result[1..z - k % 2], "\"+...");
      else
        return tostr(result, "\"");
      endif
    else
      v = type == ERR ? $code_utils:error_name(value) | tostr(value);
      len = max(4, min(max_reslen, max_toklen));
      return length(v) > len ? v[1..len - 3] + "..." | v;
    endif
    "Originally written by Mickey.";
  endverb

  verb "incr_alpha" (this none this) owner: #36 flags: "rxd"
    "args[1] is a string.  'increments' the string by one. E.g., aaa => aab, aaz => aba.  empty string => a, zzz => aaaa.";
    "args[2] is optional alphabet to use instead of $string_utils.alphabet.";
    {s, ?alphabet = this.alphabet} = args;
    index = length(s);
    if (!s)
      return alphabet[1];
    elseif (s[$] == alphabet[$])
      return this:incr_alpha(s[1..index - 1], alphabet) + alphabet[1];
    else
      t = index(alphabet, s[index]);
      return s[1..index - 1] + alphabet[t + 1];
    endif
  endverb

  verb "is_float" (this none this) owner: #36 flags: "rxd"
    "Usage:  is_float(string)";
    "Is string composed of one or more digits possibly preceded by a minus sign either followed by a decimal or by an exponent?";
    "Return true or false";
    return match(args[1], "^ *[-+]?%(%([0-9]+%.[0-9]*%|[0-9]*%.[0-9]+%)%(e[-+]?[0-9]+%)?%)%|%([0-9]+e[-+]?[0-9]+%) *$");
  endverb

  verb "inside_quotes" (this none this) owner: #36 flags: "rx"
    "Copied from Moo_tilities (#332):inside_quotes by Mooshie (#106469) Tue Dec 23 10:26:49 1997 PST";
    "Usage: inside_quotes(STR)";
    "Is the  end of the given string `inside' a doublequote?";
    "Called from $code_utils:substitute.";
    {string} = args;
    quoted = 0;
    while (i = index(string, "\""))
      if (!quoted || string[i - 1] != "\\")
        quoted = !quoted;
      endif
      string = string[i + 1..$];
    endwhile
    return quoted;
  endverb

  verb "strip_all_but_seq" (this none this) owner: #36 flags: "rxd"
    ":strip_all_but_seq(string, keep) => chars in string not in exact sequence of keep removed.";
    ":strip_all_but() works similarly, only it does not concern itself with the sequence, just the specified chars.";
    string = args[1];
    wanted = args[2];
    output = "";
    while (m = match(string, wanted))
      output = output + string[m[1]..m[2]];
      string = string[m[2] + 1..length(string)];
    endwhile
    return output;
  endverb

  verb "_tomap" (this none this) owner: #36 flags: "rxd"
    "_tomap(string) -- auxiliary for :to_value()";
    "Last modified 11/28/18 11:48 p.m. by Sinistral (#2) on ChatMUD";
    rest = this:triml(args[1]);
    vhash = [];
    if (!rest)
      return {0, []};
    elseif (rest[1] == "]")
      return {rest[2..$], []};
    endif
    while (1)
      rlen = length(rest);
      key = 0;
      if (rest[1] == "\"")
        result = this:_unquote(rest[2..rlen]);
        if (typeof(result[1] == INT))
          return result;
        endif
        key = result[2];
        rest = result[1];
        if (!rest)
          return {0, ""};
        endif
        key_end = index(rest, "->");
        if (!key_end)
          return {rlen, "missing arrow '->' in hash entry definition"};
        endif
        rest = rest[key_end + 2..$];
      elseif (w = index("{[", rest[1]))
        return {rlen, "hash key cannot be list or hash"};
      else
        key_end = index(rest, "->");
        if (!key_end)
          return {rlen, "missing arrow '->' in hash entry definition"};
        endif
        thing = rest[1..key_end - 1];
        if (typeof(s = this:_toscalar(thing)) == STR)
          return {rlen, s};
        endif
        key = s;
        rest = rest[key_end + 2..rlen];
      endif
      val = 0;
      rest = this:triml(rest);
      rlen = length(rest);
      if (w = index("{[\"", rest[1]))
        result = this:({"_tolist", "_tomap", "_unquote"}[w])(rest[2..rlen]);
        if (typeof(result[1] == INT))
          return result;
        endif
        val = result[2];
        rest = result[1];
      else
        val = rest[1..vlen = min(index(rest + ",", ","), index(rest + "]", "]")) - 1];
        if (typeof(s = this:_toscalar(val)) == STR)
          return {rlen, s};
        endif
        val = s;
        rest = rest[vlen + 1..rlen];
      endif
      vhash[key] = val;
      rest = this:triml(rest);
      if (!rest)
        return {0, vhash};
      elseif (rest[1] == "]")
        return {rest[2..$], vhash};
      elseif (rest[1] == ",")
        rest = this:triml(rest[2..$]);
      else
        return {length(rest), ", or ] expected"};
      endif
    endwhile
  endverb

  verb "autofit fit_to_screen" (this none this) owner: #36 flags: "rxd"
    ":fit_to_screen({elements}, ?padding = 2, ?underline = 0, ?separator = \" \") => Returns a columnized display that is adapted to the linelength of the viewers screen.";
    "If underline is set to 1, the first list is assumed to be a list of column headings and the code will insert an appropriate amount of dashes for you. e.g. {ur, mom, lawlz} will add {--, ---, -----}";
    "If separator is set to a character, it will be passed to :neat. e.g. those things that use lots of periods instead of spaces";
    "Verb Created by Lisdude@Toastsoft, 10/13/15";
    {elements, ?padding = 2, ?underline = 0, ?separator = " "} = args;
    ansi_utils = $ansi_utils;
    command_utils = $command_utils;
    if (underline)
      lines = {};
      for x in (elements[1])
        lines = {@lines, ansi_utils:space(length(x), "-")};
      endfor
      elements = listinsert(elements, lines, 2);
    endif
    max = $list_utils:make(length(elements[1]));
    for x in (elements)
      for y in [1..length(x)]
        if ((len = ansi_utils:length(x[y])) > max[y])
          max[y] = len;
        endif
      endfor
    endfor
    "Add padding.";
    for x in [1..length(max)]
      max[x] = max[x] + padding;
    endfor
    ret = {};
    max = this:adjust_column_lengths(max);
    for x in (elements)
      neat = {};
      for y in [1..length(x)]
        neat = {@neat, {x[y], max[y], separator}};
      endfor
      ret = {@ret, this:neat(@neat)};
    endfor
    return ret;
  endverb

  verb "adjust_column_lengths" (this none this) owner: #36 flags: "rxd"
    ":adjust_column_lengths({lengths}) => Takes a list of numbers that are assumed to be column lengths. Then, if the sum";
    "                                       of those lengths exceeds the player's linelength, beginning systematically lowering";
    "                                      individual columns until we fit on their screen.";
    {ret, ?increment = 5, ?player = player} = args;
    variable_picker = 0;
    len = length(ret);
    iterations = 0;
    linelen = player:linelen();
    while (1)
      $sin(0);
      iterations = iterations + 1;
      if (iterations >= 500000)
        break;
      endif
      sum = 0;
      for x in (ret)
        sum = sum + x;
      endfor
      if (sum <= linelen)
        return ret;
      else
        variable_picker = variable_picker + 1;
        if (variable_picker > len)
          variable_picker = len;
        endif
        if (ret[variable_picker] - increment <= 4)
          continue;
        endif
      endif
      ret[variable_picker] = ret[variable_picker] - increment;
    endwhile
    return ret;
  endverb

  verb "capitalize_each tc title_case" (this none this) owner: #36 flags: "rxd"
    "This will capitalize each word in a string.";
    "Add words to the ignore list if you want them to be left lowercase.";
    {string, ?ignore_words = {}, ?always_capitalize_first = 1} = args;
    words = $string_utils:words(string);
    for x in [1..length(words)]
      if (!(words[x] in ignore_words) || (x == 1 && always_capitalize_first))
        words[x] = $string_utils:capitalize(words[x]);
      endif
    endfor
    return $string_utils:from_list(words, " ");
  endverb

  verb "strip_binary" (this none this) owner: #2 flags: "rxd"
    {decode} = args;
    decoded = "";
    for x in (decode_binary(decode))
      if (typeof(x) == STR)
        decoded = decoded + x;
      endif
    endfor
    return decoded;
  endverb

    verb "name_of_single" (this none this) owner: #36 flags: "rxd"
    ":name_of_single(LIST objects, INT width)";
    "Similar to :names_of except this verb helps with pretty printing, it returns a list which can be columnized.";
    {items, ?width = 20} = args;
    newlist = {};
    for item in (items)
      if (typeof(item) == OBJ && valid(item))
        newlist = setadd(newlist, tostr(this:left(item.name, width * -1), " (", this:left(item, length(tostr(toint(max_object()))) + 1), ")"));
      endif
    endfor
    return newlist;
  endverb

  verb "names_of_indented" (this none this) owner: #36 flags: "rxd"
    ":names_of_indented(LIST objects) = > LIST";
    "Similar to $su:names_of except this verb helps with pretty printing, it returns a list which is indented based on the order.";
    "returns a list of strings that can be passed to :tell_lines";
    {objects} = args;
    newlist = {};
    indentsize = 0;
    for item in (objects)
      if (typeof(item) == OBJ && valid(item))
        newlist = setadd(newlist, tostr(this:left("", indentsize), "* ", this:left(item.name, -35), this:right(tostr("(", this:left(item, 5), ")"), 41 - indentsize)));
        indentsize = indentsize + 2;
      endif
    endfor
    return newlist;
  endverb

  verb "ansititle_list" (this none this) owner: #36 flags: "rxd"
    "wr_utils:title_list/title_listc(<obj-list>[, @<args>)";
    "Creates an english list out of the titles of the objects in <obj-list>.  Optional <args> are passed on to $string_utils:english_list.";
    "title_listc uses :titlec() for the first item.";
    titles = $lu:map_arg($ansi, args[2], $lu:map_verb(args[1], "title"));
    return $string_utils:english_list(titles);
  endverb

  verb "compare_sentences" (this none this) owner: #36 flags: "rxd"
    ":compare_sentence(STR original, STR second) => FLOAT difference";
    "  calculates 0-1 value on how different two sentences are.";
    "  with a 1.0, they are the same.";
    {sentence, second} = args;
    remain_s = tokens_s = $su:words(second);
    deleted = {};
    for token in (tokens_o = $su:words(sentence))
      if (i = token in remain_s)
        remain_s = listdelete(remain_s, i);
      else
        deleted = {@deleted, token};
      endif
    endfor
    difference = 1.0 - tofloat(length(deleted)) / tofloat(length(tokens_o));
    max_length = tofloat(max(length(tokens_o), length(tokens_s)));
    difference = difference - min(1.0, tofloat(length(remain_s)) / max_length);
    return max(0.0, difference);
  endverb

  verb "get_filename" (this none this) owner: #36 flags: "rxd"
    ":get_filename(STR path[, require_extension]) => STR filename";
    "  will parse out a whole path and return the filename";
    "  if no filename is found, will just return an empty string";
    {path, ?require_extension = $false} = args;
    token = `this:explode(path, "/")[$] ! E_RANGE => ""';
    if (require_extension && !("." in token))
      return "";
    endif
    return token;
  endverb

  verb "sanitize_filepath" (this none this) owner: #36 flags: "rxd"
    ":sanitize_filepath(STR subject) => STR sanitized output";
    "  adjusts filenames to not be awful for linux filesystems";
    {subject} = args;
    return pcre_replace(subject, "s/[^\\]\\[A-Za-z0-9~.,_{}\\(\\)\\'\\-\\+]/_/g");
  endverb

  verb "table" (this none this) owner: #36 flags: "rxd"
    ":table(col1, col2, col3, ..., {col1_w, col2_w, ...}[, col_Sep]) => LIST of strs";
    "  presents data in a clean table format";
    col_sep = "";
    arg_offset = 0;
    if (typeof(args[$]) == STR)
      arg_offset = 1;
      col_sep = args[$];
    endif
    cols = args[1..$ - 1 - arg_offset];
    col_widths = args[$ - arg_offset];
    results = {};
    for row in [1..max(@$lu:map_builtin(cols, "length"))]
      result = "";
      for col in [1..length(cols)]
        result = tostr(result, col != 1 ? col_sep | "", $su:left(`cols[col][row] ! E_RANGE => ""', col_widths[col]));
      endfor
      results = {@results, result};
    endfor
    return results;
  endverb

  verb "unshorten_direction" (this none this) owner: #36 flags: "rxd"
    ":unshorten_direction(STR dir) => STR longer dir";
    "  unshortens a direction";
    {dir} = args;
    return `this.exit_directions[dir] ! ANY => dir';
  endverb

  verb "progress_bar" (this none this) owner: #36 flags: "rxd"
    ":progress_bar(INT current, INT maximum, INT bar_length[, STR style, STR good_color, STR bad_color, STR suffix_str]) => STR formatted bar";
    {current, maximum, bar_length, ?style = "good_bad", ?good_color = "brgreen", ?bad_color = "brred", ?suffix_str = ""} = args;
    if (player:less_ascii())
      return tostr(tofloat(current) / tofloat(maximum), "%");
    endif
    "if the maximum is good, we want the left color to be positive";
    {colorLeft, colorRight} = style == "good_bad" ? {good_color, bad_color} | (style == "good_neutral" ? {good_color, "normal"} | (style == "bad_neutral" ? {bad_color, "normal"} | {bad_color, good_color}));
    gauge = toint(tofloat(current) / tofloat(maximum) * tofloat(bar_length));
    msg = tostr("[", $ansi:(colorLeft)($su:space(gauge, "|")), "[]", $ansi:(colorRight)($su:space(bar_length - gauge, ":")), "]", suffix_str);
    return msg;
  endverb

  verb "parse_ownership_references" (this none this) owner: #36 flags: "rxd"
    ":parse_ownership_references(string) => LIST of possible name references";
    "  i.e. :parse_ownership_references(\"Bob's cat licked Mike's dog\") => {\"Bob\", \"Mike\"}";
    {string} = args;
    names = {};
    for x in ($su:words(string))
      if ($su:endswith(x, "'s"))
        names = setadd(names, x[1..$ - 2]);
      elseif ($su:endswith(x, "s'"))
        names = setadd(names, x[1..$ - 1]);
      endif
    endfor
    return names;
  endverb

  verb "endswith" (this none this) owner: #36 flags: "rxd"
    ":endswith(string, ending) => 1 or 0";
    "  Returns 1 if the <string> ends with <ending>";
    {strg, ending} = args;
    ri = rindex(strg, ending);
    if (ri > 1)
      front = strg[1..ri - 1];
      back = strg[ri..$];
      if (back == ending)
        return 1;
      endif
    elseif (ri)
      return 1;
    endif
    return 0;
  endverb

  verb "highlight" (this none this) owner: #36 flags: "rxd"
    {string, ?color = "cyan"} = args;
    hint = player:less_ascii() ? "*" | "";
    return tostr(hint, $ansi:(color)(string));
  endverb

  verb "space" (this none this) owner: #36 flags: "rxd"
    ":space(len,fill) returns a string of length abs(len) consisting of copies of fill.  If len is negative, fill is anchored on the right instead of the left.";
    n = args[1];
    typeof(n) == STR && (n = $ansi:length(n));
    if (" " != (fill = {@args, " "}[2]))
      fill = fill + fill;
      fill = fill + fill;
      fill = fill + fill;
    elseif ((n = abs(n)) < 70)
      return "                                                                      "[1..n];
    else
      fill = "                                                                      ";
    endif
    m = (n - 1) / $ansi:length(fill);
    while (m)
      fill = fill + fill;
      m = m / 2;
    endwhile
    return n > 0 ? $ansi:cutoff(fill, 1, n) | $ansi:cutoff(fill, (f = $ansi:length(fill)) + 1 + n, f);
  endverb

  verb "left" (this none this) owner: #36 flags: "rxd"
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> followed by enough filler to make it that wide. If <width> is negative and the length of <string> is greater than the absolute value of <width>, then the <string> is cut off at <width>.";
    "";
    "The <filler> is optional and defaults to \" \"; it controls what is used to fill the resulting string when it is too short.  The <filler> is replicated as many times as is necessary to fill the space in question.";
    return z = (l = $ansi:length(out = tostr(args[1]))) < (len = abs(args[2])) ? out + this:space(l - len, length(args) >= 3 && args[3] || " ") | (args[2] > 0 ? out | $ansi:cutoff(out, 1, len));
  endverb

  verb "columnize columnise" (this none this) owner: #36 flags: "rxd"
    "columnize (items, n [, width]) - Turn a one-column list of items into an n-column list. 'width' is the last character position that may be occupied; it defaults to a standard screen width. Example: To tell the player a list of numbers in three columns, do 'player:tell_lines ($string_utils:columnize ({1, 2, 3, 4, 5, 6, 7}, 3));'.";
    items = args[1];
    n = args[2];
    width = {@args, 79}[3];
    height = (length(items) + n - 1) / n;
    items = {@items, @$list_utils:make(height * n - length(items), "")};
    colwidths = {};
    for col in [1..n - 1]
      colwidths = listappend(colwidths, 1 - (width + 1) * col / n);
    endfor
    result = {};
    for row in [1..height]
      line = tostr(items[row]);
      for col in [1..n - 1]
        line = tostr(this:left(line, colwidths[col]), " ", items[row + col * height]);
      endfor
      result = listappend(result, $ansi:cutoff(line, 1, min($ansi:length(line), width)));
    endfor
    return result;
  endverb

  verb "right" (this none this) owner: #36 flags: "rxd"
    "$su:right(string,width[,filler])";
    "Assures that <string> is at least <width> characters wide.  Returns <string> if it is at least that long, or else <string> preceded by enough filler to make it that wide. If <width> is negative and the length of <string> is greater than the absolute value of <width>, then <string> is cut off at <width>.";
    "The <filler> is optional and defaults to \" \"; it controls what is used to fill the resulting string when it is too short.  The <filler> is replicated as many times as is necessary to fill the space in question.";
    return (l = $ansi:length(out = tostr(args[1]))) < (len = abs(args[2])) ? this:space(len - l, length(args) >= 3 && args[3] || " ") + out | (args[2] > 0 ? out | $ansi:cutoff(out, 1, len));
  endverb

  verb "wrap" (this none this) owner: #36 flags: "rxd"
    "DEPRECATION INFO: this wrap is possibly a bit faster than $su:wrap but it also";
    "doesn't deal with tags as well as $su:wrap, and $su:wrap has some fixes to some";
    "long standing issues. if there is ever an issue with $su:wrap you can swap it out";
    ":old_wrap(line STR, length INT) => lines LIST";
    "takes <line> and word wraps it to <length>, returning <lines>";
    "this verb is color tag aware and will treat color tags as if they do not";
    "exist in terms of the length of the string, making it possible to wrap";
    "colored input. however, while it won't break on a color tag, it is possible";
    "it will break right after one, which can break color anyway since the end";
    "of a line will reset the colors";
    {strg, max} = args;
    "we only want to really work on this string if it's longer than max without color tags";
    without_color_tags = strg;
    if (length(without_color_tags) < max)
      "no need to bother with any of this stuff as we're shorter than the max without tags";
      return {strg};
    endif
    slist = {};
    "{'a','b','c'}";
    strg = $su:char_list(strg);
    while (length(strg) > max)
      
      newstr = "";
      "linemax defaults to the maximum";
      linemax = max;
      "check if line max is an space, if it is, we are good to break here";
      if (strg[max] != " ")
        "now we are going to backtrack through the string, looking for a space we can break";
        "on that won't break midword";
        "we will started 15 characters back or at the start of the string";
        "and walk forward";
        for findspace in [max(max - 15, 1)..min(max, length(strg))]
          "we look for a space";
          if (strg[findspace] == " ")
            "we found a space in the 15 character substring";
            linemax = findspace;
          endif
        endfor
        "at this point we will have found the furthest space in the 15 character substring";
        "that makes up the end of the string";
      endif
      for bravo in [1..linemax]
        "it's possible due to tags being treated as 'nothing' since they aren't a real 'character'";
        "that we pass the length of the string";
        if (bravo > length(strg))
          break;
        endif
        "now we continue as if we didn't encounter a color tag";
        newstr = tostr(newstr, strg[1]);
        strg = setremove(strg, strg[1]);
      endfor
      slist = {@slist, newstr};
    endwhile
    return {@slist, $su:from_list(strg)};
  endverb

  verb "proportionate" (this none this) owner: #36 flags: "rxd"
    "(INT or FLOAT total, INT or FLOAT left, INT num_messages) -> integer which is proportinate to the num_messages.";
    "(INT or FLOAT total, INT or FLOAT left, LIST of STRINGS message_list) -> proportinate message from the message_list.";
    "";
    "Returns a number > 0 or a message from the list == num_messages if total == left, == 1 ONLY if left == 0, otherwise > 1 and < num_messages as appropiate to the range.";
    "";
    "Use this to get a message from a list of proportions where the first message is the empty messages, the last message is the full message, and every message inbetween is an ammount inbetween (ie, {\"It is completely empty.\" \"It's almost empty.\" \"It's half empty.\", \"It's mostly full.\", \"It's almost full.\", \"It's completely full.\"}).";
    {total, left, third_arg} = args;
    "updated so we can tell when theres a traceback,";
    "why exactly it failed, by line number";
    if (!(typeof(total) == INT || typeof(total) == FLOAT))
      raise(E_INVARG, "Total Input is not INT or FLOAT");
    elseif (!(typeof(left) == INT || typeof(left) == FLOAT))
      raise(E_INVARG, "Left Input is not INT or FLOAT");
    elseif (!(tofloat(total) > 0.0))
      raise(E_INVARG, "Total Input is less than or equal to 0");
    elseif (!(tofloat(total) >= tofloat(left)))
      raise(E_INVARG, "Total Input is less than Left Input");
    elseif (!(tofloat(left) >= 0.0))
      raise(E_INVARG, "Left Input is less than 0");
    elseif (!(typeof(third_arg) == LIST && length(third_arg) > 0 || typeof(third_arg) == INT))
      raise(E_INVARG, "Third Arg is wrong type or empty list");
    endif
    percent = tofloat(tofloat(left) / tofloat(total) * 100.0);
    if (typeof(third_arg) == INT)
      num_messages = third_arg;
    else
      num_messages = length(third_arg);
    endif
    message_num = toint(percent / (100.0 / (tofloat(num_messages) - 1.0)) + 1.0);
    if (message_num == 1 && num_messages > 2 && tofloat(left) > 0.0)
      message_num = 2;
    endif
    if (typeof(third_arg) == INT)
      return message_num;
    else
      return third_arg[message_num];
    endif
  endverb

  verb "make_vertical" (this none this) owner: #36 flags: "rxd"
    {words, ?spacing = 1, ?indent = 0, ?bottom = 0} = args;
    max = 0;
    for x in (words)
      max = max(max, length(x));
    endfor
    if (bottom)
      for x in [1..length(words)]
        words[x] = $su:right(words[x], max);
      endfor
    endif
    line = $lu:make(max);
    for x in [1..max]
      
      line[x] = $su:space(indent, " ");
      for y in (words)
        if (length(y) < x)
          line[x] = line[x] + $su:center("", spacing);
        else
          line[x] = line[x] + $su:center(y[x], spacing);
        endif
        
      endfor
    endfor
    return line;
  endverb

  verb "starts_with ends_with" (this none this) owner: #36 flags: "rxd"
    ":starts_with(<phrase> STR, <string> STR, [<case-matters> INT]) => 1 or 0";
    ":ends_with(<phrase> STR, <string> STR, [<case-matters> INT])   => 1 or 0";
    "";
    "  Returns 1 if the <phrase> starts or ends with the <string>.";
    "  Case only matters if <case-matters> is provided and a positive value.";
    {phrase, pattern, ?casematters = 0} = args;
    if (verb == "starts_with")
      cmd = "index";
      ret = 1;
    else
      cmd = "rindex";
      if (length(phrase) < length(pattern))
        return 0;
      endif
      ret = length(phrase) + 1 - length(pattern);
    endif
    return call_function(cmd, phrase, pattern, casematters) == ret;
  endverb

  verb "ooc_message" (this none this) owner: #36 flags: "rxd"
    {string} = args;
    return tostr($ansi:green("[OOC: "), string, " ", $ansi:green("]"));
  endverb

  verb "format_float" (this none this) owner: #36 flags: "rxd"
    return tostr($mu:trunc(args[1], 2));
  endverb

  verb "is_affirmative" (this none this) owner: #36 flags: "rxd"
    ":is_affirmative(string STR) => 1 (its equal to a 'yes') or 0";
    {strg} = args;
    for x in ({"yes", "yep", "yea", "ok", "yup", "alright", "yeah", "uh-huh", "of course", "sure", "sounds great", "cool", "sounds good"})
      if (match(strg, x))
        return $true;
      endif
    endfor
    return $false;
  endverb

  verb "opposite_direction reverse_direction od" (this none this) owner: #36 flags: "rxd"
    opps = {{"east", "west"}, {"e", "west"}, {"west", "east"}, {"w", "east"}, {"north", "south"}, {"n", "south"}, {"south", "north"}, {"s", "north"}, {"northeast", "southwest"}, {"ne", "southwest"}, {"southwest", "northeast"}, {"sw", "northeast"}, {"northwest", "southeast"}, {"nw", "southeast"}, {"southeast", "northwest"}, {"se", "northwest"}, {"up", "below"}, {"u", "below"}, {"down", "above"}, {"d", "above"}, {"below", "up"}, {"in", "out"}, {"out", "in"}, {"port", "starboard"}, {"starboard", "port"}, {"star", "port"}, {"fore", "aft"}, {"aft", "fore"}};
    for x in [1..length(opps)]
      
      if ($su:match_string(args[1], opps[x][1]))
        return opps[x][2];
      endif
    endfor
    return "somewhere";
  endverb

  verb "format_value" (this none this) owner: #36 flags: "rxd"
    ":format_value(value INT|FLOAT, format STR) => display STR";
    "";
    " Formats a number in a specific IC manner according to the following:";
    "";
    " weight - integer or float value is assumed to be in grams";
    " height - integer or float value is assumed to be in centimeters";
    " value  - integer or float vlaue is assumed to be in chyen";
    {value, format, ?long = 0} = args;
    strg = "";
    if (format == "weight")
      value = tofloat(value);
      if (value > 1000.0)
        "Kgs";
        value = value / 1000.0;
        strg = "" + (long ? " kilograms" | "kg");
      else
        "grams";
        strg = "" + (long ? " grams" | "g");
      endif
      display = floatstr(value, 1);
      if (tofloat(toint(display)) == value)
        strg = tostr(toint(value)) + strg;
      else
        strg = display + strg;
      endif
    elseif (format in {"size", "volume"})
      value = tofloat(value);
      if (value > 1000000.0)
        "cubic meters";
        value = value / 1000000.0;
        strg = "" + (long ? " cubic meters" | "m^3");
      else
        "cubic centimeters";
        strg = "" + (long ? " cubic centimeters" | "cm^3");
      endif
      display = floatstr(value, 1);
      if (tofloat(toint(display)) == value)
        strg = tostr(toint(value)) + strg;
      else
        strg = display + strg;
      endif
    elseif (format == "value" || format == "worth")
      strg = $money:format(toint(value), long ? "aurum" | "aur");
    endif
    return strg;
  endverb

  verb "singularize pluralize" (this none this) owner: #36 flags: "rxd"
    {word, ?count = 1} = args;
    if (verb == "singularize")
      count = 1;
    endif
    token = $su:lowercase(word);
    if (!token)
      return token;
    endif
    "check for uncountable";
    for uncountable in ($su.UNCOUNTABLE_RULES)
      if ("$" in uncountable && pcre_match(token, uncountable))
        return token;
      elseif (token == uncountable)
        return token;
      endif
    endfor
    "check for irregulars";
    if (count == 1)
      if (maphaskey($su.IRREGULAR_SINGULAR_RULES, token))
        return token;
      elseif (new_token = `$su.IRREGULAR_PLURAL_RULES[token] ! E_RANGE => ""')
        return new_token;
      endif
    else
      if (maphaskey($su.IRREGULAR_PLURAL_RULES, token))
        return token;
      elseif (new_token = `$su.IRREGULAR_SINGULAR_RULES[token] ! E_RANGE => ""')
        return new_token;
      endif
    endif
    "singularize/pluralize";
    for replace, key in (count == 1 ? $su.SINGULARIZATION_RULES | $su.PLURALIZATION_RULES)
      new_token = pcre_replace(token, tostr("s/", key, "/", replace, "/i"));
      if (new_token != token)
        return new_token;
      endif
    endfor
    return token;
  endverb

  verb "is_plural" (this none this) owner: #36 flags: "rxd"
    {word} = args;
    token = $su:lowercase(word);
    if (!token)
      return $false;
    endif
    if (maphaskey($su.irregular_plural_rules, token))
      return $true;
    elseif (maphaskey($su.irregular_singular_rules, token))
      return $false;
    endif
    return $su:pluralize(word) != token;
  endverb

  verb "is_verb" (this none this) owner: #36 flags: "rxd"
    {word} = args;
    "remove -ing";
    if (`word[$ - 2..$] ! ANY => ""' == "ing")
      word = word[1..$ - 3];
    endif
    word = $su:singularize(word);
    return $verb_stringlist_db:find_exact(word);
  endverb

  verb "pronoun_sub_with_verbs" (this none this) owner: #36 flags: "rxd"
    {string, ?who = player, ?thing = caller, ?where = $nothing, @more_args} = args;
    if (typeof(string) == LIST)
      plines = {};
      for line in (string)
        
        plines = {@plines, this:(verb)(line, @args[2..$])};
      endfor
      return plines;
    endif
    old = tostr(string);
    new = "";
    objspec = "nditlabfgh";
    oldlen = length(old);
    while ((prcnt = index(old, "%")) && prcnt < oldlen)
      s = old[k = prcnt + 1];
      if (s == "<" && (gt = index(old[k + 2..$], ">")))
        "handling %<verb> ";
        gt = gt + k + 1;
        vb = old[k + 1..gt - 1];
        vbs = who;
        if (length(vb) > 2 && vb[2] == ":")
          " %<d:verb>";
          vbs = objects[index(objspec, vb[1]) || 1];
          vb = vb[3..$];
        endif
        vb = $object_utils:has_verb(vbs, "verb_sub") ? vbs:verb_sub(vb) | this:(verb)(vb, vbs);
        new = new + old[1..prcnt - 1] + vb;
        k = gt;
      else
        val = tostr("%", s);
        new = new + old[1..prcnt - 1] + val;
      endif
      old = old[k + 1..oldlen];
      oldlen = oldlen - k;
    endwhile
    return $su:pronoun_sub(new + old, @args[2..$]);
  endverb

  verb "shorten_direction" (this none this) owner: #36 flags: "rxd"
    {dir} = args;
    for long, short in (this.exit_directions)
      if (long == dir)
        return short;
      endif
    endfor
    return dir;
  endverb
endobject
