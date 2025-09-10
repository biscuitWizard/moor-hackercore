object #47
  name: "Recycling Center"
  parent: #5
  location: #-1
  owner: #36
  readable: true
  override "aliases" = {"Recycling Center", "Center"};

  override "description" = "";

  property "uuobj_retention_time" (owner: #36, flags: "rc") = 259200;

  property "obj_retention_time" (owner: #36, flags: "rc") = 604800;

  property "garbage" (owner: #36, flags: "rc") = [];

  property "recycling_pools" (owner: #36, flags: "rc") = {};

  verb "get_pools" (this none this) owner: #2 flags: "rxd"
    ":get_pools([OBJ for_owner]) => LIST of pools";
    "  Returns a list of all pools by default.";
    "  Otherwise if given an owner; will return all pools accessible";
    "  by that player/wiz/bit";
    {?for_owner = $nothing} = args;
    if (for_owner == $nothing)
      return this.recycling_pools;
    endif
    pools = this.recycling_pools;
    for p in (pools)
      if (!$ou:isoneof(for_owner, p.owners))
        pools = setremove(p, pools);
      endif
    endfor
    return pools;
  endverb

  verb "recycle" (this none this) owner: #2 flags: "rxd"
    ":recycle(OBJ item[, BOOL incinerate]) => NONE";
    "  Recycle is a friendly way of removing objects from the game.";
    "  Objects that are recycled will be turned into garbage objects";
    "  which can later be restored as long as the restoration attempt is";
    "  made within the retention period for that object.";
    "";
    "  An optional incinerate argument is added for objects that should";
    "  be immediately destroyed without becoming garbage.";
    {item, ?incinerate = false} = args;
    if (isplayer(item))
      raise(E_INVARG, "Recycler is unable to process player objects.");
    elseif (incinerate && !$perm_utils:controls(caller_perms(), item))
      raise(E_PERM, tostr("Unable to incinerate '", $su:nn(item), "', as caller lacks permissions."));
    endif
    this:kill_all_tasks(item);
    this:orphan(item);
    `item:recycle() ! ANY';
    if (incinerate)
      return recycle(item);
    endif
    obj_dump = dump_object(item);
    instance_id = item:instance_id();
    this.garbage[instance_id] = item;
    chparent(item, $garbage);
    item:moveto(this);
    item.disposed_on = time();
    item.object_dump = obj_dump;
    item.lifetime = $ou:is_uuobj(item) ? this.uuobj_retention_time | this.obj_retention_time;
  endverb

  verb "orphan" (this none this) owner: #2 flags: "rxd"
    ":orphan(OBJ item) => NONE";
    "  Fork that releases an object from pools";
    {item} = args;
    if ($ou:is_uuobj(item))
      "uuobjids never get pooled, so no concern.";
      return;
    endif
    fork (0)
      obj_id = toint(item);
      for pool in (this:get_pools())
        if (objid >= pool.allocation_range[1] && objid <= pool.allocation_range[2])
          pool.orphans = setadd(pool.orphans, item);
          break;
        endif
      endfor
    endfork
  endverb

  verb "create" (this none this) owner: #2 flags: "rxd"
    ":create(OBJ of_parent[, OBJ from_pool]) => Created OBJ";
    "  Used to create objects and bring them into the game!";
    "  By default :create will create only uuobjs, if a pool";
    "  is specified it will create a numbered object.";
    {of_parent, ?from_pool = $nothing} = args;
    if (from_pool == $nothing)
      new_obj = create(of_parent);
      new_obj:initialize();
      return new_obj;
    endif
    "handling pooled created for numbered objects";
    if (!$ou:isa(from_pool, $recycling_pool))
      raise (E_INVARG, tostr($su:nn(from_pool), " is not a valid $recycling_pool."));
    endif
    return from_pool:create(of_parent);
  endverb

  verb "cleanup" (this none this) owner: #2 flags: "rxd"
    ":cleanup() => LIST of incinerated instance_ids";
    "  Recycles garbage that is expired. Should be called periodically";
    "  probably by a scheduler. But can be called manually to sweep up";
    "  garbage as desired.";
    incinerated = {};
    for garbage, instance_id in (this.garbage)
      if ((garbage.disposed_on + garbage.lifetime) > time())
        continue;
      endif
      "garbage final death time";
      incinerated = setadd(incinerated, instance_id);
      this.garbage = mapdelete(this.garbage, instance_id);
      recycle(garbage);
    endfor
    return incinerated;
  endverb

  verb "valid" (this none this) owner: #2 flags: "rxd"
    ":valid(OBJ item) => True if object is valid";
    {item} = args;
    return valid(item) && parent(item) != $garbage;
  endverb

  verb "kill_all_tasks" (this none this) owner: #2 flags: "rxd"
    "kill_all_tasks ( object being recycled )";
    " -- kill all tasks involving this now-recycled object";
    caller == this || caller == #0 || raise(E_PERM);
    {object} = args;
    fork (0)
      for t in (queued_tasks())
        for c in (`task_stack(t[1]) ! E_INVARG => {}')
          if (object in c)
            kill_task(t[1]);
            continue t;
          endif
        endfor
      endfor
    endfork
  endverb
endobject
