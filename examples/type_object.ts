export type EmptyObjectType = {}
export type MyObjectType = {ident: Type, readonly ident2: number}
export type MyObjectMappedType = { [Property in keyof Type]: boolean }
