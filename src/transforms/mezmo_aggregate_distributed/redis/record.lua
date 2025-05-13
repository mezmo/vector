-- KEYS[1]: key for the ZSET tracking active windows (for expiration checks)
-- KEYS[2]: key for the event window (HASH storing aggregated values)

-- ARGV[1]: window start timestamp (milliseconds)
-- ARGV[2]: window flush timestamp (milliseconds)
-- ARGV[3]: window duration (milliseconds)
-- ARGV[4]: window cardinality limit
-- ARGV[5]: expiry grace period (milliseconds)
-- ARGV[6]: strategy (sum, avg, min, max)
-- ARGV[7]: JSON string containing unique fields from the aggregated events
-- ARGV[8]: value to aggregate

local active_windows_key = KEYS[1]
local event_window_key = KEYS[2]

local window_start_ts = tonumber(ARGV[1])
local window_flush_ts = tonumber(ARGV[2])
local window_duration_ms = tonumber(ARGV[3])
local window_cardinality_limit = tonumber(ARGV[4])
local expiry_grace_period_ms = tonumber(ARGV[5])
local strategy = ARGV[6]
local event_json = ARGV[7]
local value = tonumber(ARGV[8])

local exists = redis.call("EXISTS", event_window_key)
if exists == 0 then
  -- Check cardinality limit
  local active_window_count = redis.call("ZCARD", active_windows_key)
  if (active_window_count + 1) > window_cardinality_limit then
    return redis.error_reply("cardinality exceeded")
  end

  local window_end_ts = window_start_ts + window_duration_ms

  -- Initialize the target window
  redis.call("HSET", event_window_key,
    "strategy", strategy,
    "value", value,
    "count", 1,
    "fields", event_json,
    "window_start_ts", window_start_ts,
    "window_end_ts", window_end_ts
  )

  -- Track this window key in the set for this component, using the flush timestamp
  -- as the score. This ensures the window is kept "open" at least `window_duration_ms`
  -- from when the first event was received, rather than the end of the window. An
  -- additional, configurable grace period is added to further account for processing delays.
  redis.call("ZADD", active_windows_key, window_flush_ts, event_window_key)

  -- NOTE: until this issue is resolved, we must use EXPIREAT and EXPIRE rather than
  -- the millisecond counterparts https://github.com/dragonflydb/dragonfly/issues/4829
  local expire_secs = math.ceil((window_duration_ms + expiry_grace_period_ms) / 1000)
  local expire_ts_secs = math.ceil((window_flush_ts / 1000) + expire_secs)

  -- Ensure this window key is cleaned up if never flushed
  redis.call("EXPIREAT", event_window_key, expire_ts_secs)

  -- Also ensure the set is cleaned up, resetting the expiry of the set every
  -- time a new window is added
  redis.call("EXPIRE", active_windows_key, expire_secs, "GT")
else
  -- Update the target window based on the selected strategy
  if strategy == "min" then
    local new_value = math.min(value, tonumber(redis.call("HGET", event_window_key, "value")))
    redis.call("HSET", event_window_key, "value", new_value)
  elseif strategy == "max" then
    local new_value = math.max(value, tonumber(redis.call("HGET", event_window_key, "value")))
    redis.call("HSET", event_window_key, "value", new_value)
  else -- avg or sum
    redis.call("HINCRBYFLOAT", event_window_key, "value", value)
  end

  redis.call("HINCRBY", event_window_key, "count", 1)
end

return redis.call("HGET", event_window_key, "value")
