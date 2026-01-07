from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/git/node/section.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_view_object = resolve('view_object')
    l_0_section = resolve('section')
    l_0_side = resolve('side')
    l_0_tab = resolve('tab')
    l_0_section_change = missing
    try:
        t_1 = environment.filters['safe']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'safe' found.")
    try:
        t_2 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_2(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    l_0_section_change = context.call(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'change_stats'), 'find_change'), (undefined(name='section') if l_0_section is missing else l_0_section))
    context.vars['section_change'] = l_0_section_change
    context.exported_vars.add('section_change')
    yield '\n\n<details\n  class="diff_node"\n  '
    if (not t_2((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change))):
        pass
        yield '\n    modified="'
        yield escape((undefined(name='side') if l_0_side is missing else l_0_side))
        yield '"\n  '
    yield '\n>\n  <summary>'
    l_1_badge_text = 'S'
    pass
    template = environment.get_template('components/badge/index.jinja', 'screens/git/node/section.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'badge_text': l_1_badge_text, 'section_change': l_0_section_change}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    l_1_badge_text = missing
    yield '<span>\n      '
    yield escape((environment.getattr(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'context'), 'title_number_string') if environment.getattr(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'context'), 'title_number_string') else (Markup('&nbsp;') * ((environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'ng_level') * 2) - 1))))
    yield '\n    </span>\n    <span>'
    yield escape(environment.getattr((undefined(name='section') if l_0_section is missing else l_0_section), 'reserved_title'))
    yield '</span>'
    if ((undefined(name='tab') if l_0_tab is missing else l_0_tab) == 'diff'):
        pass
        if (not t_2((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change))):
            pass
            if (not t_2(environment.getattr((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change), 'section_token'))):
                pass
                l_1_uid = environment.getattr((undefined(name='section_change') if l_0_section_change is missing else l_0_section_change), 'section_token')
                pass
                template = environment.get_template('screens/git/sync/button.jinja', 'screens/git/node/section.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'uid': l_1_uid, 'section_change': l_0_section_change}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                l_1_uid = missing
    yield '</summary>\n\n  '
    template = environment.get_template('screens/git/fields/section_fields.jinja', 'screens/git/node/section.jinja')
    gen = template.root_render_func(template.new_context(context.get_all(), True, {'section_change': l_0_section_change}))
    try:
        for event in gen:
            yield event
    finally: gen.close()
    yield '\n</details>'

blocks = {}
debug_info = '1=28&5=32&6=35&11=40&14=48&16=50&17=52&18=54&19=56&21=60&28=68'