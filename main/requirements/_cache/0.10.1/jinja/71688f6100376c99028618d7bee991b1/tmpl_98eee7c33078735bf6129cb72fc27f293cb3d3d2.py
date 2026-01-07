from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/document/document/frame_document_content.jinja.html'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    pass
    yield '<turbo-frame id="frame_document_content">\n      <div\n        class="content"\n        data-controller="anchor_controller"\n      >\n        \n        \n\n        '
    template = environment.get_template('screens/document/document/frame_document_config.jinja.html', 'screens/document/document/frame_document_content.jinja.html')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    for (l_1_node, l_1__) in context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document_content_iterator')):
        _loop_vars = {}
        pass
        if (context.call(environment.getattr(l_1_node, 'is_section'), _loop_vars=_loop_vars) or context.call(environment.getattr(l_1_node, 'is_document'), _loop_vars=_loop_vars)):
            pass
            yield '\n          '
            template = environment.get_template('components/section/index_extends_node.jinja', 'screens/document/document/frame_document_content.jinja.html')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'_': l_1__, 'node': l_1_node}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
        elif context.call(environment.getattr(l_1_node, 'is_requirement'), _loop_vars=_loop_vars):
            pass
            yield '\n          \n            '
            template = environment.get_template('components/node_content/index_extends_node.jinja', 'screens/document/document/frame_document_content.jinja.html')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {'_': l_1__, 'node': l_1_node}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '\n          '
        else:
            pass
            yield '\n          '
            def macro():
                t_1 = []
                pass
                return concat(t_1)
            caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
            yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, False, 'Must not reach here.', caller=caller, _loop_vars=_loop_vars)
    l_1_node = l_1__ = missing
    yield '\n\n      </div>\n</turbo-frame>'

blocks = {}
debug_info = '9=13&11=19&13=22&14=25&15=31&19=34&22=44'