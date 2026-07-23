export async function executePageAction<TResult, TFailure>(
  operation: () => Promise<TResult>,
  failure: (error: unknown) => TFailure
): Promise<{ res: TResult } | TFailure> {
  try {
    return { res: await operation() }
  } catch (error) {
    return failure(error)
  }
}
