from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/document/_shared/viewtype_menu.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    pass
    def macro():
        t_1 = []
        pass
        return concat(t_1)
    caller = Macro(environment, macro, None, (), False, False, False, context.eval_ctx.autoescape)
    yield context.call(environment.extensions['strictdoc.export.html.jinja.assert_extension.AssertExtension']._assert, (not environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'standalone')), None, caller=caller)
    yield '<div class="viewtype">\n  <div\n    id="viewtype_handler"\n    class="viewtype__handler"\n    aria-expanded="false"\n    aria-controls="viewtype_menu"\n  >\n    <span>'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'get_page_title')))
    yield '</span>\n    '
    template = environment.get_template('_res/svg_ico16_expand.jinja.html', 'screens/document/_shared/viewtype_menu.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n  </div>\n  <menu\n    id="viewtype_menu"\n    class="viewtype__menu"\n    aria-hidden="true"\n    aria-labelledby="viewtype_handler"\n  >\n    <li class="viewtype__menu_header">VIEWS</li>\n    <li>\n      <a\n        data-viewtype_link="document"\n        class="viewtype__menu_item"\n        title="Go to Document view"\n        href="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_document_link'), environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), None, 'DOCUMENT'))
    yield '">\n      Document</a>\n    </li>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_table_screen')):
        pass
        yield '<li>\n      <a\n        data-viewtype_link="table"\n        class="viewtype__menu_item"\n        title="Go to Table view"\n        href="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_document_link'), environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), None, 'TABLE'))
        yield '">\n      Table</a>\n    </li>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_trace_screen')):
        pass
        yield '<li>\n      <a\n        data-viewtype_link="traceability"\n        class="viewtype__menu_item"\n        title="Go to Traceability view"\n        href="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_document_link'), environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), None, 'TRACE'))
        yield '">\n      Traceability</a>\n    </li>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_deep_trace_screen')):
        pass
        yield '<li>\n      <a\n        data-viewtype_link="deep_traceability"\n        class="viewtype__menu_item"\n        title="Go to Deep Traceability view"\n        href="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_document_link'), environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), None, 'DEEPTRACE'))
        yield '">\n      Deep Traceability</a>\n    </li>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_standalone_document')):
        pass
        yield '<li>\n      <a\n        data-viewtype_link="standalone_document"\n        class="viewtype__menu_item"\n        title="Go to Standalone Document view"\n        href="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_standalone_document_link'), environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), None))
        yield '">\n      Standalone</a>\n    </li>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_html2pdf')):
        pass
        yield '<li>\n      <a\n        data-viewtype_link="html2pdf"\n        class="viewtype__menu_item"\n        title="Go to PDF view"\n        href="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_document_link'), environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), None, 'PDF'))
        yield '">\n      PDF</a>\n    </li>'
    yield '</menu>\n</div>'

blocks = {}
debug_info = '1=12&9=19&10=21&24=28&27=30&33=33&37=35&43=38&47=40&53=43&57=45&63=48&67=50&73=53'