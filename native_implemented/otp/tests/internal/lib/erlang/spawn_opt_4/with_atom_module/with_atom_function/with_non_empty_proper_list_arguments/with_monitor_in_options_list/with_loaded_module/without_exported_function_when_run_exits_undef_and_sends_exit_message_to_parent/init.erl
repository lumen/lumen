-module(init).
-export([start/0]).
-import(erlang, [display/1]).

start() ->
  Module = erlang,
  %% Typo
  Function = '+++',
  Args = [1, 2],
  Options = [monitor],
  {ChildPid, ChildMonitorReference} = spawn_opt(Module, Function, Args, Options),
  receive
    {'DOWN', ChildMonitorReference, process, _, Info} ->
      display({child, exited, Info})
  after 10 ->
    display({child, alive, is_process_alive(ChildPid)})
  end.
