from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/project_index/actions.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    pass
    if environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_running_on_server'):
        pass
        yield '\n<turbo-frame>\n  <div class="actions_group">\n    <a\n      class="action_button"\n      href="/actions/project_index/new_document"\n      data-turbo="true"\n      title="Add new document"\n      data-testid="tree-add-document-action"\n    >'
        template = environment.get_template('_res/svg_ico16_add.jinja.html', 'screens/project_index/actions.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield ' Add document</a>'
        if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_reqif')):
            pass
            yield '<a\n        class="action_button"\n        href="/actions/project_index/import_reqif_document_form"\n        data-turbo="true"\n        title="Import document tree from ReqIF"\n        data-testid="tree-import-reqif-action"\n      >Import from ReqIF</a>'
            if (not context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'is_empty_tree'))):
                pass
                yield '<a\n          class="action_button"\n          href="/reqif/export_tree"\n          title="Export document tree to ReqIF"\n          data-testid="tree-export-reqif-action"\n        >Export to ReqIF</a>'
        yield '</div>\n</turbo-frame>'

blocks = {}
debug_info = '1=12&14=15&16=22&25=25'