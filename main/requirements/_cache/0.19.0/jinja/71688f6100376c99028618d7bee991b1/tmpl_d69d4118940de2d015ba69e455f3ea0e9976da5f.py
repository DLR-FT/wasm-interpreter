from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/anchor/anchor_clipboard_button.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_anchor_button_text = resolve('anchor_button_text')
    pass
    yield '\n<div class="anchor_button" title="Click to copy" data-testid="section-anchor-button">\n  <span class="anchor_base_icon" style="line-height: 0.1;">\n    <span class="svg_icon_not_hover_visible">'
    template = environment.get_template('_res/svg_ico16_anchor.jinja', 'components/anchor/anchor_clipboard_button.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '</span>\n    <span class="svg_icon_hover_visible">'
    template = environment.get_template('_res/svg_ico16_copy.jinja', 'components/anchor/anchor_clipboard_button.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '</span>\n  </span>\n  <span class="anchor_check_icon" style="line-height: 0.1; display: none;">\n    '
    template = environment.get_template('_res/svg_ico16_done.jinja', 'components/anchor/anchor_clipboard_button.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n  </span>\n  <span class="anchor_button_text">'
    yield escape((undefined(name='anchor_button_text') if l_0_anchor_button_text is missing else l_0_anchor_button_text))
    yield '</span>\n</div>'

blocks = {}
debug_info = '6=13&7=20&10=27&12=34'