-- KEYS[1]: key for the ZSET tracking active windows (for expiration checks)
-- KEYS[2]: key for the event window (HASH storing aggregated values)

-- ARGV[1]: window start timestamp (milliseconds)
-- ARGV[2]: window duration (milliseconds)
-- ARGV[3]: expiry grace period (milliseconds)
-- ARGV[4]: JSON string containing unique fields from the aggregated events
-- ARGV[5]: value to aggregate

local active_windows_key = KEYS[1]
local event_window_key = KEYS[2]

local window_start_ts = tonumber(ARGV[1])
local window_duration_ms = tonumber(ARGV[2])
local expiry_grace_period_ms = tonumber(ARGV[3])
local event_json = ARGV[4]
local value = tonumber(ARGV[5])

local window_end_ts = window_start_ts + window_duration_ms

-- NOTE: until this issue is resolved, we must use EXPIREAT and EXPIRE rather than
-- the millisecond counterparts https://github.com/dragonflydb/dragonfly/issues/4829
local expire_secs = math.ceil((window_duration_ms + expiry_grace_period_ms) / 1000)
local expire_ts_secs = math.ceil((window_end_ts / 1000) + expire_secs)

local exists = redis.call("EXISTS", event_window_key)
if exists == 0 then
  -- initialize the target window
  redis.call("HSET", event_window_key,
    "value", value,
    "count", 1,
    "fields", event_json,
    "window_start_ts", window_start_ts,
    "window_end_ts", window_end_ts
  )

  -- ensure this window key is cleaned up if never flushed
  redis.call("EXPIREAT", event_window_key, expire_ts_secs)

  -- track this window key in the set for this component
  redis.call("ZADD", active_windows_key, window_end_ts, event_window_key)

  -- also ensure the set is cleaned up, resetting the expiry of the set every
  -- time a new window is added
  redis.call("EXPIRE", active_windows_key, expire_secs, "GT")
else
  -- update the target window
  redis.call("HINCRBYFLOAT", event_window_key, "value", value)
  redis.call("HINCRBY", event_window_key, "count", 1)
end

return redis.call("HGET", event_window_key, "value")
