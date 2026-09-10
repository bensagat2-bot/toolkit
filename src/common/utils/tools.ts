const MAX_NAME_LENGTH = 80
const MAX_FILE_NAME_LENGTH = 150

export const clipNameLength = (name: string) => {
  if (name.length <= MAX_NAME_LENGTH || !name.includes('、')) return name
  const names = name.split('、')
  let newName = names.shift()!
  for (const name of names) {
    if (newName.length + name.length > MAX_NAME_LENGTH) break
    newName = newName + '、' + name
  }
  return newName
}

export const clipFileNameLength = (name: string) => {
  return name.length > MAX_FILE_NAME_LENGTH ? name.substring(0, MAX_FILE_NAME_LENGTH) : name
}
