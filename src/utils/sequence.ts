import {AsyncSequence} from '../generated/sequence.async'
import {Sequence} from '../generated/sequence.normal'

export {Sequence}
export {AsyncSequence}

type AsyncIterableOrIterator<T> = AsyncIterable<T> | AsyncIterator<T>
type IterableOrIterator<T> = Iterable<T> | Iterator<T>

export function asSequence<T>(iterator: IterableOrIterator<T>): Sequence<T> {
  let gotIterator: Iterator<T> | undefined = undefined
  return new Sequence<T>({
    next: (): IteratorResult<T> => {
      if (!gotIterator) {
        if (Symbol.iterator in iterator)
          gotIterator = callIterable<Iterator<T>>(iterator, Symbol.iterator)
        else
          gotIterator = iterator as Iterator<T>
      }
      return gotIterator.next()
    },
  })
}

export function asAsyncSequence<T>(
  iterator: AsyncIterableOrIterator<T>,
): AsyncSequence<T> {
  let gotIterator: AsyncIterator<T> | undefined = undefined
  return new AsyncSequence<T>({
    next: async (): Promise<IteratorResult<T>> => {
      if (!gotIterator) {
        if (Symbol.asyncIterator in iterator)
          gotIterator = callIterable<AsyncIterator<T>>(iterator, Symbol.asyncIterator)
        else if (Symbol.iterator in iterator)
          gotIterator = callIterable<AsyncIterator<T>>(iterator, Symbol.iterator)
        else
          gotIterator = iterator as AsyncIterator<T>
      }
      return await gotIterator.next()
    },
  })
}

function callIterable<I>(iterableOrIterator: unknown, key: symbol | string): I {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  return iterableOrIterator[key]()
}
