from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'screens/document/_shared/toc.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_last_moved_node_id = resolve('last_moved_node_id')
    l_0_view_object = resolve('view_object')
    try:
        t_1 = environment.filters['safe']
    except KeyError:
        @internalcode
        def t_1(*unused):
            raise TemplateRuntimeError("No filter named 'safe' found.")
    try:
        t_2 = environment.tests['defined']
    except KeyError:
        @internalcode
        def t_2(*unused):
            raise TemplateRuntimeError("No test named 'defined' found.")
    try:
        t_3 = environment.tests['none']
    except KeyError:
        @internalcode
        def t_3(*unused):
            raise TemplateRuntimeError("No test named 'none' found.")
    pass
    yield '    <ul\n      data-testid="toc-list"\n      js-collapsible_list="list"\n      class="toc"\n      id="toc"\n      data-controller="draggable_list"'
    if t_2((undefined(name='last_moved_node_id') if l_0_last_moved_node_id is missing else l_0_last_moved_node_id)):
        pass
        yield '\n      data-last_moved_node_id="'
        yield escape(context.call(environment.getattr((undefined(name='last_moved_node_id') if l_0_last_moved_node_id is missing else l_0_last_moved_node_id), 'get_string_value')))
        yield '"\n      '
    yield '>'
    l_1_loop = missing
    for (l_1_section, l_1__), l_1_loop in LoopContext(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'table_of_contents')), undefined):
        l_1_range = resolve('range')
        _loop_vars = {}
        pass
        yield '<li data-nodeid="'
        yield escape(context.call(environment.getattr(environment.getattr(l_1_section, 'reserved_mid'), 'get_string_value'), _loop_vars=_loop_vars))
        yield '">'
        if context.call(environment.getattr(l_1_section, 'is_section'), _loop_vars=_loop_vars):
            pass
            if ((not environment.getattr(l_1_section, 'ng_has_requirements')) and context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'is_deeptrace'), _loop_vars=_loop_vars)):
                pass
                yield '<span class="toc-title-no-link" title="Section has no requirements">\n              <span class="section-number">\n                '
                yield escape(environment.getattr(environment.getattr(l_1_section, 'context'), 'title_number_string'))
                yield '\n              </span>'
                yield escape(environment.getattr(l_1_section, 'title'))
                yield '\n            </span>\n          '
            else:
                pass
                yield '<a\n              href="#'
                yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_local_anchor'), l_1_section, _loop_vars=_loop_vars))
                yield '"\n              anchor="'
                yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_local_anchor'), l_1_section, _loop_vars=_loop_vars))
                yield '"\n              data-turbo="false"\n            >\n              <span class="section-number">\n                '
                yield escape((environment.getattr(environment.getattr(l_1_section, 'context'), 'title_number_string') if environment.getattr(environment.getattr(l_1_section, 'context'), 'title_number_string') else (Markup('&nbsp;') * ((environment.getattr(l_1_section, 'ng_level') * 2) - 1))))
                yield '\n              </span>'
                yield escape(environment.getattr(l_1_section, 'title'))
                yield '\n              \n            </a>'
        else:
            pass
            yield '\n        <a\n          href="#'
            yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_local_anchor'), l_1_section, _loop_vars=_loop_vars))
            yield '"\n          anchor="'
            yield escape(context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'render_local_anchor'), l_1_section, _loop_vars=_loop_vars))
            yield '"\n          data-turbo="false"\n        >\n          <span class="section-number">\n            '
            yield escape((environment.getattr(environment.getattr(l_1_section, 'context'), 'title_number_string') if environment.getattr(environment.getattr(l_1_section, 'context'), 'title_number_string') else (Markup('&nbsp;') * ((environment.getattr(l_1_section, 'ng_level') * 2) - 1))))
            yield '\n          </span>'
            if (not t_3(environment.getattr(l_1_section, 'reserved_title'))):
                pass
                yield escape(environment.getattr(l_1_section, 'reserved_title'))
                yield '\n            '
            yield '</a>'
        if (not environment.getattr(l_1_loop, 'last')):
            pass
            if (environment.getattr(environment.getitem(environment.getattr(l_1_loop, 'nextitem'), 0), 'ng_level') > environment.getattr(l_1_section, 'ng_level')):
                pass
                yield '<ul>'
            elif (environment.getattr(environment.getitem(environment.getattr(l_1_loop, 'nextitem'), 0), 'ng_level') < environment.getattr(l_1_section, 'ng_level')):
                pass
                yield '</li>'
                for l_2_x in context.call((undefined(name='range') if l_1_range is missing else l_1_range), 0, (environment.getattr(l_1_section, 'ng_level') - environment.getattr(environment.getitem(environment.getattr(l_1_loop, 'nextitem'), 0), 'ng_level')), _loop_vars=_loop_vars):
                    _loop_vars = {}
                    pass
                    yield '</ul>\n            </li>'
                l_2_x = missing
            else:
                pass
                yield '</li>'
        else:
            pass
            yield '</li>'
            for l_2_x in context.call((undefined(name='range') if l_1_range is missing else l_1_range), 0, (environment.getattr(l_1_section, 'ng_level') - 1), _loop_vars=_loop_vars):
                _loop_vars = {}
                pass
                yield '</ul>\n            </li>'
            l_2_x = missing
            yield '</ul>'
    l_1_loop = l_1_section = l_1__ = l_1_range = missing

blocks = {}
debug_info = '7=32&8=35&11=39&12=44&13=46&15=48&18=51&19=53&31=58&32=60&36=62&37=64&45=69&46=71&50=73&52=75&53=77&60=80&61=82&63=85&65=88&74=99'