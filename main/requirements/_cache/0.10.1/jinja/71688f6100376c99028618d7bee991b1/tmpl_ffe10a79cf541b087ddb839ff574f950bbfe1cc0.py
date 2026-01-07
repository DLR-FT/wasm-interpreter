from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/document/_shared/project_tree.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    pass
    if (not context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'is_empty_tree'))):
        pass
        yield '<div class="tree">\n    '
        for l_1_folder_or_file in context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'iterator_files_first')):
            l_1_document_ = resolve('document_')
            _loop_vars = {}
            pass
            if context.call(environment.getattr(l_1_folder_or_file, 'is_folder'), _loop_vars=_loop_vars):
                pass
                yield '\n        '
                if context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'should_display_folder'), l_1_folder_or_file, _loop_vars=_loop_vars):
                    pass
                    yield '\n          <div\n            class="tree_folder"\n            data-level="'
                    yield escape(environment.getattr(l_1_folder_or_file, 'level'))
                    yield '"\n            data-testid="tree-folder-item"\n          >\n          <span class="tree_folder_path">/'
                    yield escape(environment.getattr(l_1_folder_or_file, 'rel_path'))
                    yield '</span>\n          </div>\n        '
                yield '\n      '
            elif context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'should_display_file'), l_1_folder_or_file, _loop_vars=_loop_vars):
                pass
                l_1_document_ = context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'get_document_by_path'), environment.getattr(l_1_folder_or_file, 'full_path'), _loop_vars=_loop_vars)
                _loop_vars['document_'] = l_1_document_
                yield '\n        '
                if (not context.call(environment.getattr((undefined(name='document_') if l_1_document_ is missing else l_1_document_), 'document_is_included'), _loop_vars=_loop_vars)):
                    pass
                    yield '\n        <a\n          href="'
                    yield escape(context.call(environment.getattr(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'meta'), 'get_root_path_prefix'), _loop_vars=_loop_vars))
                    yield '/'
                    yield escape(context.call(environment.getattr(environment.getattr((undefined(name='document_') if l_1_document_ is missing else l_1_document_), 'meta'), 'get_html_doc_link'), _loop_vars=_loop_vars))
                    yield '"\n          class="tree_item"\n          '
                    if (environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document') == (undefined(name='document_') if l_1_document_ is missing else l_1_document_)):
                        pass
                        yield '\n          active="true"\n          '
                    yield '\n          data-folder="'
                    yield escape(environment.getattr(l_1_folder_or_file, 'mount_folder'))
                    yield '"\n          data-testid="tree-document-link"\n        >\n          '
                    template = environment.get_template('_res/svg_ico16_document.jinja.html', 'screens/document/_shared/project_tree.jinja')
                    gen = template.root_render_func(template.new_context(context.get_all(), True, {'document_': l_1_document_, 'folder_or_file': l_1_folder_or_file}))
                    try:
                        for event in gen:
                            yield event
                    finally: gen.close()
                    yield '\n          <div\n            class="document_title"\n            title="'
                    yield escape(environment.getattr((undefined(name='document_') if l_1_document_ is missing else l_1_document_), 'title'))
                    yield '"\n            data-file_name="'
                    yield escape(environment.getattr(l_1_folder_or_file, 'file_name'))
                    yield '"\n          >'
                    yield escape(environment.getattr((undefined(name='document_') if l_1_document_ is missing else l_1_document_), 'title'))
                    yield '</div>\n        </a>\n        '
                    l_2_document = (undefined(name='document_') if l_1_document_ is missing else l_1_document_)
                    pass
                    yield '\n        '
                    template = environment.get_template('screens/document/_shared/project_tree_child_documents.jinja', 'screens/document/_shared/project_tree.jinja')
                    gen = template.root_render_func(template.new_context(context.get_all(), True, {'document': l_2_document, 'document_': l_1_document_, 'folder_or_file': l_1_folder_or_file}))
                    try:
                        for event in gen:
                            yield event
                    finally: gen.close()
                    yield '\n        '
                    l_2_document = missing
                    yield '\n        '
                yield '\n      '
        l_1_folder_or_file = l_1_document_ = missing
        yield '</div>'
    else:
        pass
        yield '<span data-testid="document-tree-empty-text">🐛 The project has no documents yet.</span>'

blocks = {}
debug_info = '2=12&4=15&5=19&6=22&9=25&14=27&17=30&18=32&19=35&21=38&23=42&26=46&29=48&32=55&33=57&34=59&37=64'