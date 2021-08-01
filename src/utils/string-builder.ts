export class StringBuilder {
  body: string

  constructor() {
    this.body = ""
  }

  ln(line: string): void {
    this.body += `${line}\n`
  }

  append(elem: string): this {
    this.body += elem
    return this
  }

  toString(): string {
    return this.body
  }
}
