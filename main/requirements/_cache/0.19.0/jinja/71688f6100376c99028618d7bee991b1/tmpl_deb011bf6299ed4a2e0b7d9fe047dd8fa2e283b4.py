from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/document/document/action_list.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    pass
    if (not environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'standalone')):
        pass
        yield '<a\n  href="/actions/document/edit_grammar?document_mid='
        yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
        yield '"\n  class="dropdown_menu_item"\n  data-turbo="true"\n  data-turbo-action="replace"\n  title="Edit document grammar"\n  data-testid="document-edit-grammar-action"\n  role="menuitem"\n>'
        template = environment.get_template('_res/svg_ico16_gear.jinja.html', 'screens/document/document/action_list.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield ' Edit grammar</a>'
        if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_html2pdf')):
            pass
            yield '<a\n  class="dropdown_menu_item"\n  href="/export_html2pdf/'
            yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
            yield '"\n  data-testid="document-export-html2pdf-action"\n  role="menuitem"\n>'
            template = environment.get_template('_res/svg_ico16_export.jinja', 'screens/document/document/action_list.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield ' Export to PDF</a>'
        if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_reqif')):
            pass
            yield '<a\n  class="dropdown_menu_item"\n  href="/reqif/export_document/'
            yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
            yield '"\n  data-testid="document-export-reqif-action"\n  role="menuitem"\n>'
            template = environment.get_template('_res/svg_ico16_export.jinja', 'screens/document/document/action_list.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield ' Export to ReqIF</a>'

blocks = {}
debug_info = '1=12&3=15&10=17&12=24&15=27&18=29&21=36&24=39&27=41'