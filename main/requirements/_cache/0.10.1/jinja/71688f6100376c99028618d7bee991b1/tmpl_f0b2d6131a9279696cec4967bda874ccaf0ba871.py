from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/document/traceability/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    parent_template = None
    pass
    parent_template = environment.get_template('base.jinja.html', 'screens/document/traceability/index.jinja')
    for name, parent_block in parent_template.blocks.items():
        context.blocks.setdefault(name, []).append(parent_block)
    yield from parent_template.root_render_func(context)

def block_head_css(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_super = context.super('head_css', block_head_css)
    _block_vars = {}
    l_0_view_object = resolve('view_object')
    pass
    yield '\n  '
    yield escape(context.call(l_0_super, _block_vars=_block_vars))
    yield '\n  <link rel="stylesheet" href="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'requirement-tree.css', _block_vars=_block_vars))
    yield '"/>\n'

def block_head_scripts(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_super = context.super('head_scripts', block_head_scripts)
    _block_vars = {}
    l_0_view_object = resolve('view_object')
    pass
    yield '\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'stimulus_umd.min.js', _block_vars=_block_vars))
    yield '"></script>\n  <script>\n    Stimulus.application = Stimulus.Application.start();\n  </script>\n\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'viewtype_menu.js', _block_vars=_block_vars))
    yield '"></script>\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'resizable_bar.js', _block_vars=_block_vars))
    yield '"></script>\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'collapsible_toc.js', _block_vars=_block_vars))
    yield '"></script>\n  <script src="'
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'toc_highlighting.js', _block_vars=_block_vars))
    yield '"></script>'
    if (environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_running_on_server') and (not environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'standalone'))):
        pass
        yield '<script type="module">\n    import hotwiredTurbo from "'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'turbo.min.js', _block_vars=_block_vars))
        yield '";\n  </script>\n  <script src="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'controllers/modal_controller.js', _block_vars=_block_vars))
        yield '"></script>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_mathjax'), _block_vars=_block_vars):
        pass
        yield '<script id="MathJax-script" async src="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'mathjax/tex-mml-chtml.js', _block_vars=_block_vars))
        yield '"></script>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_rapidoc'), _block_vars=_block_vars):
        pass
        yield '<script src="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'rapidoc/rapidoc-min.js', _block_vars=_block_vars))
        yield '"></script>'
    if context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'project_config'), 'is_activated_mermaid'), _block_vars=_block_vars):
        pass
        yield '<script src="'
        yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_static_url'), 'mermaid/mermaid.min.js', _block_vars=_block_vars))
        yield '"></script>\n    <script type="module">\n      mermaid.initialize({ startOnLoad: true });\n    </script>'
    yield escape(context.call(l_0_super, _block_vars=_block_vars))
    yield '\n'

def block_title(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    l_0_view_object = resolve('view_object')
    pass
    yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'title'))
    yield ' - '
    yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'get_page_title'), _block_vars=_block_vars))

def block_viewtype(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield 'traceability'

def block_layout_nav(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    l_0_view_object = resolve('view_object')
    pass
    if (not environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'standalone')):
        pass
        template = environment.get_template('_shared/nav.jinja.html', 'screens/document/traceability/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()

def block_tree_content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\n  '
    template = environment.get_template('screens/document/_shared/resizable_bar_with_project_tree.jinja', 'screens/document/traceability/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n'

def block_toc_content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\n  '
    template = environment.get_template('screens/document/_shared/resizable_bar_with_toc.jinja', 'screens/document/traceability/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n'

def block_header_content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    l_1_header__items = ['screens/document/_shared/frame_header_document_title.jinja', 'screens/document/_shared/viewtype_menu.jinja']
    pass
    template = environment.get_template('components/header/index.jinja', 'screens/document/traceability/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'header__items': l_1_header__items}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    l_1_header__items = missing

def block_main_content(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    _block_vars = {}
    pass
    yield '\n  '
    template = environment.get_template('screens/document/traceability/main.jinja', 'screens/document/traceability/index.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n'

blocks = {'head_css': block_head_css, 'head_scripts': block_head_scripts, 'title': block_title, 'viewtype': block_viewtype, 'layout_nav': block_layout_nav, 'tree_content': block_tree_content, 'toc_content': block_toc_content, 'header_content': block_header_content, 'main_content': block_main_content}
debug_info = '1=12&3=17&4=28&5=30&8=33&9=44&14=46&15=48&16=50&17=52&19=54&21=57&23=59&26=61&27=64&30=66&31=69&34=71&35=74&40=76&42=79&43=92&45=102&46=111&47=113&51=120&52=129&55=137&56=146&59=154&66=164&70=172&71=181'