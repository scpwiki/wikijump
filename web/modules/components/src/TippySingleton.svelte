<script lang="ts">
  import { onDestroy } from "svelte"
  import tippy, {
    createSingleton,
    type CreateSingletonProps,
    type Instance,
    type Props
  } from "tippy.js"
  import { DEFAULT_TIPPY_OPTS, parseTipOpts } from "./lib/tippy"

  const DEFAULT_SINGLETON_PROPS: Partial<CreateSingletonProps> = {
    ...DEFAULT_TIPPY_OPTS
  }

  export let opts: Partial<CreateSingletonProps> = { ...DEFAULT_SINGLETON_PROPS }

  let singleton = createSingleton([])
  const tips = new Set<Instance>()

  function update() {
    if (singleton) {
      singleton.setProps({ ...DEFAULT_SINGLETON_PROPS, ...opts })
      singleton.setInstances([...tips])
    }
  }

  function tip(elem: Element, opts: Partial<Props> | string = "") {
    opts = parseTipOpts(elem, opts)
    const tp = tippy(elem, opts)
    const setState = (content: unknown) => {
      if (!content) tp.disable()
      else tp.enable()
    }
    setState(opts.content)

    tips.add(tp)
    update()

    return {
      update(opts: Partial<Props> | string = "") {
        opts = parseTipOpts(elem, opts)
        tp.setProps(opts)
        setState(opts.content)
        update()
      },
      destroy() {
        tips.delete(tp)
        tp.destroy()
        update()
      }
    }
  }

  onDestroy(() => {
    singleton.destroy()
    // @ts-ignore
    singleton = undefined
  })
</script>

<slot {tip} />
