local start = 0
local finish = 110000
local missing = {}

for i=start,finish do
  local exists = redis.call('EXISTS', i)
  if exists == 0 then
    table.insert(missing, i)
  end
end

return missing