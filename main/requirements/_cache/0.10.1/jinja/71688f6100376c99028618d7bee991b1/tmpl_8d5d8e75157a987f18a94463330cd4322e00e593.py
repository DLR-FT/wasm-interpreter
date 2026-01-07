from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/project_index/project_map_file.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    l_0_file = resolve('file')
    l_0_document = missing
    pass
    l_0_document = context.call(environment.getattr(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'traceability_index'), 'document_tree'), 'get_document_by_path'), environment.getattr((undefined(name='file') if l_0_file is missing else l_0_file), 'full_path'))
    context.vars['document'] = l_0_document
    context.exported_vars.add('document')
    if (not context.call(environment.getattr((undefined(name='document') if l_0_document is missing else l_0_document), 'document_is_included'))):
        pass
        yield '\n "'
        yield escape(context.call(environment.getattr(environment.getattr((undefined(name='document') if l_0_document is missing else l_0_document), 'meta'), 'get_html_doc_link')))
        yield '": [\n'
        l_1_section = (undefined(name='document') if l_0_document is missing else l_0_document)
        pass
        template = environment.get_template('screens/project_index/project_map_section.jinja', 'screens/project_index/project_map_file.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {'section': l_1_section, 'document': l_0_document}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        l_1_section = missing
        yield ' ],'

blocks = {}
debug_info = '1=14&2=17&3=20&5=24'