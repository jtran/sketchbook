OP_SET = 0
OP_COPY = 1
OP_PUSH = 2
OP_POP = 3
OP_DELETE_KEY = 4
OP_COMMENT = 5

OP_CODES = {}
OP_CODES[OP_SET] = 'OP_SET'
OP_CODES[OP_COPY] = 'OP_COPY'
OP_CODES[OP_PUSH] = 'OP_PUSH'
OP_CODES[OP_POP] = 'OP_POP'
OP_CODES[OP_DELETE_KEY] = 'OP_DELETE_KEY'
OP_CODES[OP_COMMENT] = 'OP_COMMENT'

inspectObject = (obj) ->
  if obj == undefined
    'undefined'
  else if obj == null
    'null'
  else if Array.isArray(obj)
    parts = for v in obj
      inspectObject(v)
    "[#{parts.join(', ')}]"
  else if typeof(obj) == 'object'
    if obj.inspect?
      obj.inspect()
    else
      parts = for k, v of obj
        "#{k}: #{inspectObject(v)}"
      if parts.length > 0
        "{ #{parts.join(', ')} }"
      else
        "{}"
  else if typeof(obj) == 'string'
    '"' + obj.replace(/"/g, '\\"') + '"'
  else
    '' + obj

appendElement = (parentElem, elem) ->
  parentElem.appendChild(elem)
  # Without the timeout, the opacity gets set without transitioning.
  setTimeout((=> elem.style.opacity = 1), 10)

removeFromLayout = (value) ->
  return unless value?
  style = value.elem.style
  # Remove left and top from the transition.
  style.transition = 'opacity 0.5s ease-out'
  style.position = 'absolute'
  style.left = '5px'
  style.top = '5px'

class DomView
  constructor: (className = null) ->
    @elem = document.createElement('div')
    @elem.className = 'node'
    @elem.classList.add(className) if className

  addViewClass: (className) ->
    @elem.classList.add(className)

  detachFromView: ->
    @elem?.remove()

class Cell extends DomView

  constructor: (value = null, className = null) ->
    super('cell')
    @value = value
    if className?
      @addViewClass(className)

class Prim extends DomView
  constructor: (value) ->
    super()
    @value = value
    @elem.innerText = @inspect()

  isPrimitive: -> true

  copyPrimitive: -> new Prim(@value)

  inspect: -> '' + @value

class Arr extends DomView
  constructor: (array) ->
    super('array')
    @array = array[..]

    @detailsElem = document.createElement('div')
    @detailsElem.className = 'node'
    @updateDetails()
    appendElement(@elem, @detailsElem)

    @cells = []
    for obj in array
      cell = new Cell(obj)
      appendElement(cell.elem, obj.elem)
      appendElement(@elem, cell.elem)
      @cells.push(cell)

  updateDetails: ->
    @detailsElem.innerText = "Array(length=#{@array.length})"

  isPrimitive: -> false

  getLength: -> @array.length

  push: (val) ->
    @array.push(val)
    console.assert(not val.elem.parentElement, "You're attaching a node to an array view, but the node is already attached: ", val, val.elem.parentElement)
    @updateDetails()
    cell = new Cell(val)
    @cells.push(cell)
    appendElement(cell.elem, val.elem)
    appendElement(@elem, cell.elem)

  pop: ->
    val = @array.pop()
    cell = @cells.pop()
    cell?.detachFromView()
    @updateDetails()
    val

  getIndex: (index) ->
    @array[index]

  setIndex: (index, value) ->
    previousVal = @array[index]
    removeFromLayout(previousVal)
    @array[index] = value
    cell = @cells[index]
    appendElement(cell.elem, value.elem)

    previousVal

  inspect: -> '[' + @array.map((obj) -> obj.inspect()).join(', ') + ']'

class MapEntry

  constructor: (@key, @value) ->
    @keyCell = keyCell = new Cell(@key, 'map-key')
    @valCell = valCell = new Cell(@key, 'map-val')
    keyCell.elem.innerText = '' + @key
    appendElement(valCell.elem, @value.elem)
    @children = [keyCell, valCell]

  setValue: (@value) ->
    appendElement(@valCell.elem, @value.elem)

  appendTo: (parentElem) ->
    for obj in @children
      appendElement(parentElem, obj.elem)

  detachFromView: ->
    for obj in @children
      obj.detachFromView()

class MapValue extends DomView
  constructor: (map) ->
    super('map')
    @map = new Map(map)

    @detailsElem = document.createElement('div')
    @detailsElem.className = 'node map-header'
    @updateDetails()
    appendElement(@elem, @detailsElem)

    @entries = new Map()
    for [key, val] from @map
      entry = new MapEntry(key, val)
      entry.appendTo(@elem)
      @entries.set(key, entry)

  updateDetails: ->
    @detailsElem.innerText = "Map(size=#{@map.size})"

  isPrimitive: -> false

  getSize: -> @map.size

  deleteKey: (key) ->
    val = @map.get(key)
    @map.delete(key)
    entry = @entries.get(key)
    @entries.delete(key)
    entry?.detachFromView()
    @updateDetails()
    val

  getIndex: (key) ->
    @map.get(key)

  setIndex: (key, value) ->
    previousVal = @map.get(key)
    if previousVal?
      removeFromLayout(previousVal)
      @map.set(key, value)
      entry = @entries.get(key)
      if not entry?
        throw new Error("I expected an entry to exist for an existing key in the map, but I found none: #{inspectObject(key)}")
      entry.setValue(value)
    else
      @map.set(key, value)
      console.assert(not value.elem.parentElement, "You're attaching a node to a map view, but the node is already attached: ", value, value.elem.parentElement)
      @updateDetails()
      entry = new MapEntry(key, value)
      entry.appendTo(@elem)
      @entries.set(key, entry)

    previousVal

  inspect: -> '{' + ("#{key} => #{val.inspect()}" for [key, val] from @map.entries()).join(', ') + '}'

class Ref extends DomView
  constructor: (to) ->
    super()
    @to = to
    @elem.innerText = @inspect()
    @pathElem = document.createElementNS('http://www.w3.org/2000/svg', 'path')

  # Primitives are things that take up a single word of memory and can be copied
  # in a single operation.
  isPrimitive: -> true

  copyPrimitive: -> new Ref(@to)

  inspect: -> "<ref #{@to.id}>"

  detachFromView: ->
    if @pathElem
      @pathElem.remove()
      @pathElem = null
    super()

class EmptyVal extends DomView

  constructor: ->
    super()
    @elem.innerText = @inspect()

  isPrimitive: -> true

  copyPrimitive: -> new EmptyVal()

  inspect: -> '<undefined>'

# A location in memory that can be read from and written to.  This is sometimes
# called an l-value.
class Location

  VAR_PATTERN = /^([a-zA-Z_][a-zA-Z_0-9]*)(.*)/

  @parse: (locStr) ->
    if locStr.length == 0
      return null

    match = VAR_PATTERN.exec(locStr)
    if not match?
      return null
    loc = new VarLocation(match[1])
    locStr = match[2]
    loop
      [index, locStr] = @parseIndex(locStr)
      if not index?
        break
      loc = new IndexLocation(loc, index)
    loc

  INDEX_PATTERN = /^\s*\[\s*([^\]\s]*)\s*\](.*)/

  @parseIndex: (locStr) ->
    match = INDEX_PATTERN.exec(locStr)
    if not match
      return [null, locStr]
    indexStr = match[1]
    locStr = match[2]
    indexHostVal = JSON.parse(indexStr)
    if typeof(indexHostVal) not in ['number', 'string']
      throw new Error("Expected array index to be numeric or double-quoted string, but found: #{indexStr}")

    [indexHostVal, locStr]

class VarLocation

  constructor: (@id) ->

  inspect: -> "<VarLocation id=#{@id}>"

class IndexLocation

  constructor: (@loc, @index) ->

  inspect: -> "<IndexLocation loc=#{@loc.inspect()}, index=#{JSON.stringify(@index)}>"

class HeapEntry

  constructor: (@key) ->
    level = 0
    @keyElem = document.createElement('div')
    @keyElem.className = 'entry-key'
    @keyElem.innerText = '' + @key
    @valElem = document.createElement('div')
    @valElem.className = 'entry-value cell'
    @elem = document.createElement('div')
    @elem.className = 'entry'
    @indexIntoArr = null
    @altElem = null
    appendElement(@elem, @keyElem)
    appendElement(@elem, @valElem)

    @value = null

  setValue: (val) ->
    removeFromLayout(@value)
    @value = val
    appendElement(@valElem, val.elem)
    if @indexIntoArr?
      @updateAltView()
    val

  setLevel: (@level) ->

  # Set the alternate view as the index into an array.  Param should be the
  # the array.
  setIndexInto: (arr) ->
    if @altElem?
      @altElem.remove()
      @altElem = null
    @indexIntoArr = arr
    @altElem = document.createElement('div')
    @altElem.className = 'node index-into'
    @altElem.style.left = '-140px'
    @altElem.style.width = '125px'
    appendElement(arr.elem, @altElem)
    if @value?
      @updateAltView()

  updateAltView: ->
    style = @altElem.style
    indexHeight = 28
    originTop = 20
    intVal = if @value instanceof Prim and typeof(@value.value) == 'number'
      @value.value
    else
      null
    if intVal?
      len = @indexIntoArr.getLength()
      if intVal < 0
        top = originTop - indexHeight
      else if intVal < len
        top = originTop + intVal * indexHeight
      else
        top = originTop + len * indexHeight
      style.top = top + 'px'
      style.opacity = 1
      @altElem.innerText = "#{@key} = #{intVal}"
    else
      @altElem.innerText = ''
      style.opacity = 0

  detachFromView: ->
    @elem?.remove()
    @altElem?.remove()

class Interpreter
  constructor: ->
    @ops = []
    @heap = {}
    @nextId = 0
    @elem = null
    @levelElems = []
    @allRefs = new Map()

  attachTo: (@elem) ->
    for i in [0 ... 5] by 1
      elem = document.createElement('div')
      elem.className = if i == 0 then 'level first' else 'level'
      @levelElems.push(elem)
      @elem.appendChild(elem)

  run: (ops) ->
    for op in ops
      @runOp(op)

  humanOp: (op) ->
    [opCode, args...] = op
    # Convert op code to string.
    [OP_CODES[opCode], args...]

  # Convert an op to a human readable string.
  humanDisplay: (op) ->
    parts = switch op[0]
      when OP_SET
        [op[1], ':=', inspectObject(op[2]), op[3..]...]
      when OP_COPY
        [op[1], '<-', op[2..]...]
      when OP_PUSH
        [OP_CODES[op[0]], op[1], inspectObject(op[2]), op[3..]...]
      when OP_POP, OP_DELETE_KEY
        [OP_CODES[op[0]], op[1..]...]
      else
        [OP_CODES[op[0]], op[1..]...]
    # Strip off outer array.
    parts.join(' ')

  addOp: (op) ->
    @ops.push(op)

  runOp: (op) ->
    console.log @humanOp(op)...
    @addOp(op)
    [opCode, args...] = op
    needsRedraw = true
    switch opCode
      when OP_SET
        dest = Location.parse(args[0])
        val = @hostToNewObject(args[1])
        options = @parseOpOptions(args[2..])
        if val.isPrimitive()
          previousVal = @write dest, val, options
        else
          loc = new VarLocation(@alloc())
          @write(loc, val, heapLevel: @heapLevel(dest) + 1)
          previousVal = @write dest, @createRef(loc), options
        previousVal?.detachFromView()
      when OP_COPY
        dest = Location.parse(args[0])
        srcVal = @read(Location.parse(args[1]))
        copyVal = @copyPrimitive(srcVal)
        previousVal = @write dest, copyVal
        @transition copyVal, srcVal, =>
          previousVal?.detachFromView()
      when OP_PUSH
        arrLoc = Location.parse(args[0])
        obj = @deref(@read(arrLoc))
        if not obj.push?
          throw Error("Runtime Error: Can't push onto non-array")
        val = @hostToNewObject(args[1])
        if val.isPrimitive()
          valToPush = val
        else
          loc = new VarLocation(@alloc())
          @write(loc, val, heapLevel: @heapLevel(arrLoc) + 1)
          valToPush = @createRef(loc)
        obj.push(valToPush)
      when OP_POP
        arrLoc = Location.parse(args[0])
        obj = @deref(@read(arrLoc))
        if not obj.pop?
          throw Error("Runtime Error: Can't pop from non-array")
        obj.pop()
      when OP_DELETE_KEY
        mapLoc = Location.parse(args[0])
        mapObj = @deref(@read(mapLoc))
        if not mapObj.deleteKey?
          throw Error("Runtime Error: Can't delete key from non-map value")
        mapObj.deleteKey(args[1])
      when OP_COMMENT
        needsRedraw = false
        console.log(args...)

    if needsRedraw
      @redrawRefs()

    @logState()

  parseOpOptions: (args) ->
    opts = {}
    if args.length <= 0
      return opts

    kv = /^([a-zA-Z_]+)=([^\s]*)/
    for arg in args
      match = kv.exec(args[0])
      if match?
        switch match[1]
          when 'index_into'
            opts[match[1]] = Location.parse(match[2])
          else
            opts[match[1]] = match[2]

    opts

  read: (loc) ->
    if loc instanceof VarLocation
      entry = @heap[loc.id]
      return entry?.value
    else if loc instanceof IndexLocation
      objVal = @deref(@read(loc.loc))
      return objVal.getIndex(loc.index)
    else
      throw new Error("Unknown location type while trying to read: #{loc}")

  readEntry: (loc) ->
    if loc instanceof VarLocation
      @heap[loc.id]
    else if loc instanceof IndexLocation
      objEntry = @derefEntry(@read(loc.loc))
      objVal = objEntry?.value
      return null unless objVal?
      indexVal = objVal.getIndex(loc.index)
      if indexVal instanceof Ref
        @derefEntry(indexVal)
      else
        objEntry
    else
      throw new Error("Unknown location type while trying to read entry: #{inspectObject(loc)}")

  write: (loc, val, options = {}) ->
    if loc instanceof VarLocation
      entry = @heap[loc.id]
      if not entry?
        entry = new HeapEntry(loc.id)
        indexIntoLoc = options['index_into']
        if indexIntoLoc?
          entry.setIndexInto(@deref(@read(indexIntoLoc)))
        @heap[loc.id] = entry
        heapLevel = options.heapLevel ? @heapLevel(loc)
        entry.setLevel(heapLevel)
        levelIndex = Math.min(heapLevel, @levelElems.length - 1)
        appendElement(@levelElems[levelIndex], entry.elem)
      else if entry.value == val
        # Same exact object.  Nothing to do.
        return null

      previousVal = entry.value
      entry.setValue(val)
    else if loc instanceof IndexLocation
      objVal = @deref(@read(loc.loc))
      previousVal = objVal.setIndex(loc.index, val)
    else
      throw new Error("Unknown location type while writing value to it: #{inspectObject(loc)}")

    @updateRefArrow(val, previousVal)

    previousVal

  # If it's an ID like _0, _1, _2, etc., then we allocated it.
  _HEAP_PATTERN = /^_[\d]+$/

  heapLevel: (loc) ->
    entry = @readEntry(loc)

    entry?.level ? 0

  # Allocate a new block of memory and return its address.
  alloc: ->
    @_genId()

  deref: (ref) ->
    @read(ref.to)

  derefEntry: (ref) ->
    @readEntry(ref.to)

  createRef: (loc) ->
    unless loc instanceof VarLocation
      throw new Error("createRef expected a VarLocation but found: #{loc}")
    ref = new Ref(loc)
    # Save all refs by what they point to.
    arr = @allRefs.get(loc.id)
    if not arr?
      arr = []
      @allRefs.set(loc.id, arr)
    # TODO: This is a memory leak.  We don't currently remove them.
    arr.push(ref)

    ref

  redrawRefs: ->
    for [key, refs] from @allRefs
      for ref in refs
        @updateRefArrow(ref)

  updateRefArrow: (val, previousValue = null) ->
    if previousValue instanceof Ref
      prevArrowElem = previousValue.arrowElem
      if prevArrowElem?
        prevArrowElem.style.opacity = '0'
        setTimeout =>
          prevArrowElem.remove()
        , 500

    if val not instanceof Ref
      return

    path = val.pathElem
    if not path?
      # Reference has been detached.  Nothing to do.
      return

    obj = @deref(val)
    toPos = @position(obj)
    toPos[0] += -10
    toPos[1] += 8

    fromObj = val
    fromPos = @position(fromObj)
    fromPos[0] += 10 + fromObj.elem.offsetWidth
    fromPos[1] += 8
    console.log(fromObj.elem)

    path.setAttribute('d', "M#{fromPos[0]} #{fromPos[1]} L#{toPos[0]} #{toPos[1]}")

    g = @elem.querySelector('svg g')
    g.appendChild(path)

  hostToNewObject: (val) ->
    if Array.isArray(val)
      new Arr(val.map((v) => @hostToNewObject(v)))
    else if not val?
      new EmptyVal()
    else if typeof(val) == 'object'
      map = new Map()
      for k, v of val
        map.set(k, @hostToNewObject(v))
      new MapValue(map)
    else
      new Prim(val)

  _genId: ->
    id = @nextId
    # TODO: Check for overflow.
    @nextId++
    "_#{id}"

  assertPrimitive: (val) ->
    unless val.isPrimitive()
      throw Error("You tried to copy a non-primitive; the reference should be copied instead")

  copyPrimitive: (val) ->
    @assertPrimitive(val)
    val.copyPrimitive()

  logState: ->
    for k,entry of @heap
      console.log entry.key, entry.value.inspect()

  transition: (val, from, callback = null) ->
    pos = @position(from, relativeTo: val)
    @setPosition val, [pos[0] + 5, pos[1] + 5], =>
      @setPosition val, [0, 0], =>
        callback?()
    return

  position: (val, options = {}) ->
    { relativeTo = null } = options
    # Walk up the offset parents.
    obj = val.elem
    x = 0
    y = 0
    while obj? and obj != @elem
      x += obj.offsetLeft
      y += obj.offsetTop
      obj = obj.offsetParent

    if not relativeTo?
      # console.log(x, y)
      return [x, y]

    origin = @position(relativeTo)
    # console.log(x, y, origin[0], origin[1])

    [x - origin[0], y - origin[1]]

  setPosition: (val, pos, callback = null) ->
    [x, y] = pos
    # console.log "setPosition", val, x, y
    style = val.elem.style
    style.left = x + 'px'
    style.top = y + 'px'
    if callback?
      setTimeout (=> callback()), 500
    pos

interpreter = new Interpreter()
window.rt = interpreter

# interpreter.runOp([OP_SET, 'n', 3])
# interpreter.runOp([OP_COPY, 'm', 'n'])
# interpreter.runOp([OP_SET, 'a', []])
# interpreter.runOp([OP_PUSH, 'a', 1])
# interpreter.runOp([OP_PUSH, 'a', 2])
# interpreter.runOp([OP_PUSH, 'a', 3])
# interpreter.runOp([OP_POP, 'a'])
# interpreter.runOp([OP_COPY, 'b', 'a'])

ops = []
window.ops = ops

# fact = (n) ->
#   stack = []          ; ops.push([OP_SET, 'stack', []])
#   while n >= 1
#     stack.push(n)     ; ops.push([OP_PUSH, 'stack', n])
#     n--
#   r = 1               ; ops.push([OP_SET, 'r', 1])
#   while stack.length > 0
#     x = stack.pop()   ; ops.push([OP_POP, 'stack'])
#     r = r * x         ; ops.push([OP_SET, 'r', r])
#   r

# Model the call stack.
# ops.push([OP_SET, 'stack', []])
# fact = (n) ->
#   ops.push([OP_PUSH, 'stack', n])
#   if n <= 1
#     r = 1
#   else
#     r = n * fact(n - 1)
#   ops.push([OP_SET, 'r', r])
#   ops.push([OP_POP, 'stack'])
#   r

# x = 4
# console.log("fact(#{x})", fact(x))

# Swap
# ops.push([OP_SET, 'a', 1])
# ops.push([OP_SET, 'b', 2])
# ops.push([OP_COPY, 'temp', 'b'])
# ops.push([OP_COPY, 'b', 'a'])
# ops.push([OP_COPY, 'a', 'temp'])

# Test that references work.
# ops.push([OP_SET, 'a', [1, 2, 3]])
# ops.push([OP_SET, 'b', [4, 5, 6]])
# ops.push([OP_COPY, 'c', 'a'])
# # This should update `a`.
# ops.push([OP_SET, 'c[0]', 0])

# Test that maps work.
testMaps = ->
  ops.push([OP_SET, 'm', {}])
  ops.push([OP_SET, 'm[2]', 'b'])
  ops.push([OP_SET, 'm[3]', 'c'])
  ops.push([OP_COPY, 'x', 'm[2]'])
  ops.push([OP_SET, 'y', 7])
  ops.push([OP_COPY, 'm[3]', 'y'])
  ops.push([OP_DELETE_KEY, 'm', 2])

# Map of arrays.
testMapOfArrays = ->
  ops.push([OP_SET, 'animals', {}])
  ops.push([OP_SET, 'animals["bird"]', []])
  ops.push([OP_SET, 'animals["fox"]', []])
  ops.push([OP_SET, 'animals["wolf"]', []])
  ops.push([OP_PUSH, 'animals["bird"]', "bluejay"])
  ops.push([OP_PUSH, 'animals["bird"]', "falcon"])
  ops.push([OP_PUSH, 'animals["bird"]', "robin"])
  ops.push([OP_PUSH, 'animals["fox"]', "arctic fox"])
  ops.push([OP_PUSH, 'animals["fox"]', [1, 2, 3]])
  ops.push([OP_PUSH, 'animals["fox"]', "kit fox"])
  ops.push([OP_PUSH, 'animals["fox"]', "red fox"])
  ops.push([OP_PUSH, 'animals["wolf"]', "arctic wolf"])
  ops.push([OP_PUSH, 'animals["wolf"]', "eastern wolf"])
  ops.push([OP_PUSH, 'animals["wolf"]', "gray wolf"])

selectionSort = (arr, sortBy = (x) -> x) ->
  len = arr.length                   ; ops.push([OP_SET, 'len', len])
  for i in [0 ... len - 1] by 1
    ops.push([OP_SET, 'i', i])

    minIndex = i                     ; ops.push([OP_COPY, 'minIndex', 'i'])
    for j in [i + 1 ... len] by 1
      ops.push([OP_SET, 'j', j, 'index_into=arr'])

      if sortBy(arr[j]) < sortBy(arr[minIndex])
        minIndex = j                 ; ops.push([OP_COPY, 'minIndex', 'j'])
    # Swap.
    tmp = arr[i]           ; ops.push([OP_COPY, 'tmp', "arr[#{i}]"])
    arr[i] = arr[minIndex] ; ops.push([OP_COPY, "arr[#{i}]", "arr[#{minIndex}]"])
    arr[minIndex] = tmp    ; ops.push([OP_COPY, "arr[#{minIndex}]", 'tmp'])

testSelectionSortNumbers = ->
  arr = [3, 5, 0, 6, 9, 4]
  ops.push([OP_SET, 'arr', []])
  for x in arr
    ops.push([OP_PUSH, 'arr', x])
  # Sort it.
  selectionSort(arr)
  console.log('arr after sort', arr)
testSelectionSortNumbers()

testSelectionSortObjects = ->
  # Build the array.
  arr = [
    {
      name: 'fox'
      score: 3
    }
    {
      name: 'falcon'
      score: 5
    }
    {
      name: 'wolf'
      score: 0
    }
    {
      name: 'canary'
      score: 6
    }
    {
      name: 'coyote'
      score: 9
    }
    {
      name: 'eagle'
      score: 4
    }
  ]
  ops.push([OP_SET, 'arr', []])
  for x in arr
    ops.push([OP_PUSH, 'arr', x])
  # Sort it.
  selectionSort(arr, (obj) -> obj.score)
  console.log('arr after sort', arr)

###########################################################
# The UI.

opIndex = 0
scrubber = document.getElementById('scrubber')

displayOps = ->
  lastOpElem = document.getElementById('last-op')
  nextOpElem = document.getElementById('next-op')
  lastOpElem.innerText = nextOpElem.innerText
  if opIndex < ops.length
    # Strip off outer array.
    nextOpElem.innerText = interpreter.humanDisplay(ops[opIndex])
  else
    nextOpElem.innerText = '<end of ops>'

advanceOp = ->
  return false if opIndex >= ops.length

  op = ops[opIndex]
  interpreter.runOp(op)
  opIndex++
  scrubber.value = opIndex
  displayOps()
  true

runToEnd = ->
  # Run to the end of all ops.  Don't pause after each step.
  null while advanceOp()
  return

interpreter.attachTo(document.getElementById('interpreter'))
document.getElementById('advance').addEventListener 'click', (event) ->
  event.preventDefault()
  advanceOp()
  return

document.getElementById('advance-ten').addEventListener 'click', (event) ->
  event.preventDefault()
  for i in [1..10]
    break if not advanceOp()
  return

runAllStep = ->
  advanceOp()
  setTimeout(runAllStep, 1500)

document.getElementById('run-all').addEventListener 'click', (event) ->
  event.preventDefault()
  runAllStep()
  return

main = ->
  if ops.length > 0
    scrubber.setAttribute('max', ops.length - 1)
  displayOps()

window.advanceOp = advanceOp
window.runToEnd = runToEnd

main()
