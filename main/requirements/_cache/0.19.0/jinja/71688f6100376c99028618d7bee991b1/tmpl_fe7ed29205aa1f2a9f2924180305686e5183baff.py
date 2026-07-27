from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/node/copy_stable_link_button.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_path = resolve('path')
    try:
        t_1 = environment.tests['defined']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No test named 'defined' found.")
    pass
    if t_1((undefined(name='path') if l_0_path is missing else l_0_path)):
        pass
        yield '<div\n    data-controller="copy_stable_link_button"\n    data-path="'
        yield escape((undefined(name='path') if l_0_path is missing else l_0_path))
        yield '"\n    class="copy_stable_link-button"\n    title="Click to copy a stable node link to the clipboard"\n  >\n    <span style="display: contents;" class="copy_to_clipboard-copy_icon">\n      '
        template = environment.get_template('_res/svg_ico16_link.jinja', 'components/node/copy_stable_link_button.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    </span>\n    <span style="display: none;" class="copy_to_clipboard-done_icon">\n      '
        template = environment.get_template('_res/svg_ico16_done.jinja', 'components/node/copy_stable_link_button.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '\n    </span>\n  </div>'

blocks = {}
debug_info = '4=18&7=21&12=23&15=30'