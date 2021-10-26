// about formatted strings

// get list of the character after %
type FormatChars<T extends string> =
// eslint-disable-next-line @typescript-eslint/no-unused-vars
  T extends `${infer _}%`
    ? never
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    : T extends `${infer _}%${infer Format}${infer Tail}`
      ? Format extends '%'
        ? FormatChars<Tail>
        : [Format, ...FormatChars<Tail>]
      : []

// apply FormatValue for each T[n]
type FormatValues<Map, T extends string[]> =
  T extends [] ? []
  : T extends [infer Head, ...infer Tail]
    ? Head extends string
      ? Tail extends string[]
        ? [FormatValue<Map, Head>, ...FormatValues<Map, Tail>]
        : never
      : never
    : never

// returns type of Map[T] and never if not found
type FormatValue<Map, T extends string> = T extends keyof Map ? Map[T] : never

// same as FormatValues<Map, FormatChars<T>> but returns [never]
// if FormatValues<Map, FormatChars<T>> is never
export type FormatArgs<Map, T extends string> =
  FormatValues<Map, FormatChars<T>> extends never
    ? [never]
    : FormatValues<Map, FormatChars<T>>

export type FormatProcessor<Map> = {
  [P in keyof Map]: (value: Map[P]) => string
}

export type Awaitable<T> = T | PromiseLike<T>
