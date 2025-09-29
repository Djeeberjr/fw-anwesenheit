local used_ids = {}
local id_file_path = "used_ids.txt"

local function load_ids()
  local file = io.open(id_file_path, "r")
  if not file then return end
  for line in file:lines() do
    used_ids[line:lower()] = true
  end
  file:close()
end

local function save_id(id)
  local file = io.open(id_file_path, "a")
  if file then
    file:write(id:lower() .. "\n")
    file:close()
  end
end

local function gen_id()
  local id = ""
  for i = 1, 10 do
    id = id .. string.format("%x", math.random(0, 15))
  end
  return id
end

local function get_new_id()
  local tries = 0
  while tries < 10000 do
    local id = gen_id()
    if not used_ids[id:lower()] then
      return id
    end
    tries = tries + 1
  end
  error("Could not generate a new unused ID after 10000 tries")
end

local function write_new_card()
  local id = get_new_id()
  local cmd = string.format("lf em 410x clone --id %s", id)
  core.console(cmd)
  used_ids[id:lower()] = true
  save_id(id)
  print("Wrote new EM4100 card with ID:", id)
end

local function write_new_card(id)
  local cmd = string.format("lf em 410x clone --id %s", id)
  core.console(cmd)
  used_ids[id:lower()] = true
  save_id(id)
  print("Wrote new EM4100 card with ID:", id)
end

math.randomseed(os.time())
load_ids()
local id = get_new_id()
write_new_card(id)
