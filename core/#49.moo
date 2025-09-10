object #49
  name: "Generic Garbage"
  location: #-1
  owner: #36
  parent: #1
  readable: true
  override "aliases" = {"Garbage"};

  override "description" = "Object reuse. Call $recycler:_create() to create an object (semantics the same as create()), $recycler:_recycle() to recycle an object. Will create a new object if nothing available in its contents. Note underscores, to avoid builtin :recycle() verb called when objects are recycled. Uses $building_utils:recreate() to prepare objects.";

  property "object_dump" (owner: #36, flags: "r") = {};

  property "disposed_on" (owner: #36, flags: "r") = 0;

  property "lifetime" (owner: #36, flags: "r") = 0;
  
endobject
