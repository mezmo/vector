-- Implements a sliding window rate-limiter with support for multiple active windows.
--
-- KEYS[1]: key for the ZSET tracking active windows
-- KEYS[2]: key for the event window
local active_windows_key = KEYS[1]
local event_window_key = KEYS[2]

-- ARGV[1]: threshold of max events per-window
-- ARGV[2]: window duration (milliseconds)
-- ARGV[3]: `now()` timestamp (milliseconds)
-- ARGV[4]: cardinality limit for all active windows
local threshold = tonumber(ARGV[1])
local window_duration_ms = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local window_cardinality_limit = tonumber(ARGV[4])

local ALLOWED = 1
local DISALLOWED = 0

-- retain only active windows, freeing available slots for this call
redis.call("ZREMRANGEBYSCORE", active_windows_key, "-inf", now - window_duration_ms)
redis.call("ZREMRANGEBYSCORE", event_window_key, "-inf", now - window_duration_ms)

local window_is_active = redis.call("ZSCORE", active_windows_key, event_window_key)
if not window_is_active then
  -- adding a new window is subject to cardinality check
  local active_window_count = redis.call("ZCARD", active_windows_key)
  if (active_window_count + 1) > window_cardinality_limit then
    return redis.error_reply("cardinality exceeded")
  end
end

-- upsert the event window and reset the expiry of the active window set
redis.call("ZADD", active_windows_key, now, event_window_key)
redis.call("PEXPIRE", active_windows_key, window_duration_ms)

local count = redis.call("ZCARD", event_window_key)
if count < threshold then
  redis.call("ZADD", event_window_key, now, now)
  redis.call("PEXPIRE", event_window_key, window_duration_ms)
  return ALLOWED
else
  return DISALLOWED
end
