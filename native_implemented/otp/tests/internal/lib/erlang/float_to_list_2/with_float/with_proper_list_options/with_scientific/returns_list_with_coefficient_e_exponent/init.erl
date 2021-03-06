-module(init).
-export([start/0]).
-import(erlang, [append_element/2, display/1, float_to_list/2, make_tuple/2]).

start() ->
  lists().

lists() ->
  %% FIXME 0-10 to work around https://github.com/lumen/lumen/issues/512
   lists(0, 10).

lists(MaxDigits, MaxDigits) ->
  list(MaxDigits);
lists(Digits, MaxDigits) ->
  list(Digits),
  lists(Digits + 1, MaxDigits).

list(Digits) ->
  Scientific = pair(scientific, Digits),
  Options = [Scientific],
  display(float_to_list(1234567890.0987654321, Options)).

%% FIXME work around tuple lowering bug
pair(Key, Value) ->
  Empty = make_tuple(0, []),
  KeyTuple = append_element(Empty, Key),
  append_element(KeyTuple, Value).
