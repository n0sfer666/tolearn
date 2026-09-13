export const refusal = (code, message) => ({ code, message });
export const rejected = (failure) => () => Promise.reject(failure);
export const held = () => new Promise(() => {});
export const press = (host, selector) => host.querySelector(selector).click();
export const named = (calls, name) => calls.filter((made) => made.name === name);
export const began = (step, round = 0, of = 0) => ({ step, state: "began", round, of });

export function deferred() {
  let settle = { resolve: () => {}, reject: () => {} };
  const promise = new Promise((resolve, reject) => {
    settle = { resolve, reject };
  });
  return {
    answer: () => promise,
    resolve: (value) => settle.resolve(value),
    reject: (failure) => settle.reject(failure),
  };
}

export function transport(answers) {
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    const answer = answers[name];
    if (answer === undefined) throw new Error(`лишняя команда ${name}`);
    return typeof answer === "function" ? answer(payload) : Promise.resolve(answer);
  };
  return { calls, call };
}

export function heard() {
  let emit = () => {};
  let stops = 0;
  const steps = (handler) => {
    emit = handler;
    return () => {
      stops += 1;
      emit = () => {};
    };
  };
  return { steps, emit: (step) => emit(step), stops: () => stops };
}

export function sequence(...answers) {
  let turn = 0;
  return (payload) => {
    const answer = answers[Math.min(turn, answers.length - 1)];
    turn += 1;
    return answer(payload);
  };
}
