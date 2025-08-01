-- KEYS[1]: key for active windows
---
-- TODO(mdeltito): this list MUST be provided rather than computed or evaluated within
-- this scripts.
--
-- With hash tags, we can be sure that all bucket keys are in the same slot (component_id).
-- However, by default Dragonfly disables the Redis option which allows for accessing undeclared keys
-- https://github.com/dragonflydb/dragonfly/issues/272#issuecomment-2019767858
--
-- This breaks the atomicity of this operation, but in theory it should not
-- matter as the next call will flush values that arrive slightly "late" vs. the
-- zrange call for expired buckets.
-- KEYS[2..N]: keys for expired windows to flush
local active_windows_key = KEYS[1]

local numeric_fields = { value = true, count = true, window_start_ts = true, window_end_ts = true }
local results = {}

-- flush all expired buckets
for i = 2, #KEYS do
  local bucket_key = KEYS[i]
  local hash_data = redis.call("HGETALL", bucket_key)

  if #hash_data > 0 then
    local bucket = {}

    for j = 1, #hash_data, 2 do
      local key = hash_data[j]
      local value = hash_data[j + 1]

      if numeric_fields[key] then
        bucket[key] = tonumber(value)
      else
        bucket[key] = value
      end
    end

    table.insert(results, bucket)
  end

  -- unset flushed bucket state
  redis.call("DEL", bucket_key)
  redis.call("ZREM", active_windows_key, bucket_key)
end

if #results == 0 then
  -- return empty array if no results. unlike the openresty fork of cjson,
  -- the bundled cjson lib does not have the ability to mark a
  -- table explicitly as an array. when there are no entries in the
  -- table it will encode as a map instead of a sequence.
  return "[]"
else
  return cjson.encode(results)
end
