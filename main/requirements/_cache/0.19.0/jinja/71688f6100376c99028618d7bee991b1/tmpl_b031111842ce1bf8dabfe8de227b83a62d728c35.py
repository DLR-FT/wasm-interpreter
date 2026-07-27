from jinja2.runtime import LoopContext, Macro, Markup, Namespace, TemplateNotFound, TemplateReference, TemplateRuntimeError, Undefined, escape, identity, internalcode, markup_join, missing, str_join
name = 'components/node/node_controls/index.jinja'

def root(context, missing=missing):
    resolve = context.resolve_or_missing
    undefined = environment.undefined
    concat = environment.concat
    cond_expr_undefined = Undefined
    if 0: yield None
    l_0_sdoc_entity = resolve('sdoc_entity')
    l_0_view_object = resolve('view_object')
    pass
    yield '\n\n\n<sdoc-node-controls>\n\n  '
    if (context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')) == 'document'):
        pass
        yield '\n  <a\n    href="/actions/document/edit_config?document_mid='
        yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
        yield '"\n    class="action_button"\n    data-turbo-action="replace"\n    data-turbo="true"\n    title="Edit title and meta"\n    data-testid="document-edit-config-action"\n  >'
        template = environment.get_template('_res/svg_ico16_edit.jinja.html', 'components/node/node_controls/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '</a>'
    if (context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')) != 'document'):
        pass
        if (not environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'is_root_included_document')):
            pass
            yield '\n  <a\n    href="/actions/document/edit_'
            yield escape(context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')))
            yield '?node_id='
            yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
            yield '&context_document_mid='
            yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
            yield '"\n    class="action_button"\n    data-turbo-action="replace"\n    data-turbo="true"\n    title="Edit"\n    data-testid="node-edit-action"\n  >'
            template = environment.get_template('_res/svg_ico16_edit.jinja.html', 'components/node/node_controls/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '</a>\n  <a\n    href="/actions/document/delete_'
            yield escape(context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')))
            yield '?node_id='
            yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
            yield '&context_document_mid='
            yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
            yield '"\n    class="action_button"\n    data-turbo-method="delete"\n    data-turbo="true"\n    title="Delete"\n    data-testid="node-delete-action"\n  >'
            template = environment.get_template('_res/svg_ico16_delete.jinja.html', 'components/node/node_controls/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '</a>\n  '
        else:
            pass
            yield '\n    <a\n      href="/actions/document/edit_included_document?document_mid='
            yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
            yield '&context_document_mid='
            yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
            yield '"\n      class="action_button"\n      data-turbo-action="replace"\n      data-turbo="true"\n      title="Edit"\n      data-testid="node-edit-action"\n    >'
            template = environment.get_template('_res/svg_ico16_edit.jinja.html', 'components/node/node_controls/index.jinja')
            gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
            try:
                for event in gen:
                    yield event
            finally: gen.close()
            yield '</a>\n  '
        yield '\n'
    yield '\n\n  '
    if (context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')) == 'requirement'):
        pass
        yield '<a\n    href="/actions/document/clone_requirement?reference_mid='
        yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
        yield '&context_document_mid='
        yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
        yield '"\n    class="action_button"\n    \n    \n    data-turbo="true"\n    title="Clone"\n    data-testid="node-clone-action"\n  >'
        template = environment.get_template('_res/svg_ico16_copy.jinja', 'components/node/node_controls/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '</a>'
    if ((context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')) != 'document') or (not environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'section_contents'))):
        pass
        yield '\n  \n  <sdoc-menu\n    js-dropdown-menu\n    data-controller="dropdown_menu"\n    class="add_node"\n  >\n  \n    <sdoc-menu-handler\n      js-dropdown-menu-handler\n      aria-expanded="false"\n    >\n      <a\n        class="action_button"\n        title="Add node"\n        data-testid="node-menu-handler"\n      >'
        template = environment.get_template('_res/svg_ico16_add.jinja.html', 'components/node/node_controls/index.jinja')
        gen = template.root_render_func(template.new_context(context.get_all(), True, {}))
        try:
            for event in gen:
                yield event
        finally: gen.close()
        yield '</a>\n    </sdoc-menu-handler>\n\n    <sdoc-menu-list>\n      <menu\n        js-dropdown-menu-list\n        aria-hidden="true"\n      >\n        <header>Add node</header>\n\n        '
        if (context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')) == 'document'):
            pass
            yield '\n\n          '
            for l_1_element_ in context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'get_grammar_elements')):
                _loop_vars = {}
                pass
                yield '\n            \n            <li class="sdoc-menu-actions-group">'
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield ':</li>\n\n            \n            <li></li>\n            <li></li>\n            <li>\n              <a\n                  href="/actions/document/new_requirement?reference_mid='
                yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
                yield '&whereto=child&element_type='
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '&context_document_mid='
                yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
                yield '"\n                  data-turbo-action="replace"\n                  data-turbo="true"\n                  title="Add first child '
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '"\n                  data-testid="node-add-'
                yield escape(context.call(environment.getattr(l_1_element_, 'get_tag_lower'), _loop_vars=_loop_vars))
                yield '-first-action"\n                >'
                template = environment.get_template('_res/svg_ico16_add_child.jinja', 'components/node/node_controls/index.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'element_': l_1_element_}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                yield '</a>\n            </li>'
            l_1_element_ = missing
        if (context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')) == 'section'):
            pass
            yield '\n          '
            for l_1_element_ in context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'get_grammar_elements')):
                _loop_vars = {}
                pass
                yield '\n            <li class="sdoc-menu-actions-group">'
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield ':</li>\n\n            \n            <li>\n              <a\n                href="/actions/document/new_requirement?reference_mid='
                yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
                yield '&whereto=before&element_type='
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '&context_document_mid='
                yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
                yield '"\n                data-turbo-action="replace"\n                data-turbo="true"\n                title="Add '
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield ' above"\n                data-testid="node-add-'
                yield escape(context.call(environment.getattr(l_1_element_, 'get_tag_lower'), _loop_vars=_loop_vars))
                yield '-above-action"\n              >'
                template = environment.get_template('_res/svg_ico16_add_above.jinja', 'components/node/node_controls/index.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'element_': l_1_element_}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                yield '</a>\n            </li>\n            <li>\n              <a\n                  href="/actions/document/new_requirement?reference_mid='
                yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
                yield '&whereto=child&element_type='
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '&context_document_mid='
                yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
                yield '"\n                  data-turbo-action="replace"\n                  data-turbo="true"\n                  title="Add child '
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '"\n                  data-testid="node-add-'
                yield escape(context.call(environment.getattr(l_1_element_, 'get_tag_lower'), _loop_vars=_loop_vars))
                yield '-child-action"\n                >'
                template = environment.get_template('_res/svg_ico16_add_child.jinja', 'components/node/node_controls/index.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'element_': l_1_element_}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                yield '</a>\n            </li>\n            <li>\n              <a\n                href="/actions/document/new_requirement?reference_mid='
                yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
                yield '&whereto=after&element_type='
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '&context_document_mid='
                yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
                yield '"\n                data-turbo-action="replace"\n                data-turbo="true"\n                title="Add '
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield ' below"\n                data-testid="node-add-'
                yield escape(context.call(environment.getattr(l_1_element_, 'get_tag_lower'), _loop_vars=_loop_vars))
                yield '-below-action"\n              >'
                template = environment.get_template('_res/svg_ico16_add_below.jinja', 'components/node/node_controls/index.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'element_': l_1_element_}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                yield '</a>\n            </li>\n          '
            l_1_element_ = missing
            yield '\n\n        '
        if (context.call(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'get_type_string')) == 'requirement'):
            pass
            yield '\n          '
            for l_1_element_ in context.call(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'get_grammar_elements')):
                _loop_vars = {}
                pass
                yield '\n            <li class="sdoc-menu-actions-group">'
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield ':</li>\n\n            \n            '
                if (not environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'is_composite')):
                    pass
                    yield '\n              <li></li>\n            '
                yield '\n            <li>\n              <a\n                href="/actions/document/new_requirement?reference_mid='
                yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
                yield '&whereto=before&element_type='
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '&context_document_mid='
                yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
                yield '"\n                data-turbo-action="replace"\n                data-turbo="true"\n                title="Add '
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield ' above"\n                data-testid="node-add-'
                yield escape(context.call(environment.getattr(l_1_element_, 'get_tag_lower'), _loop_vars=_loop_vars))
                yield '-above-action"\n              >'
                template = environment.get_template('_res/svg_ico16_add_above.jinja', 'components/node/node_controls/index.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'element_': l_1_element_}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                yield '</a>\n            </li>\n            '
                if environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'is_composite'):
                    pass
                    yield '\n            <li>\n              <a\n                href="/actions/document/new_requirement?reference_mid='
                    yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
                    yield '&whereto=child&element_type='
                    yield escape(environment.getattr(l_1_element_, 'tag'))
                    yield '&context_document_mid='
                    yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
                    yield '"\n                data-turbo-action="replace"\n                data-turbo="true"\n                title="Add child '
                    yield escape(environment.getattr(l_1_element_, 'tag'))
                    yield '"\n                data-testid="node-add-'
                    yield escape(context.call(environment.getattr(l_1_element_, 'get_tag_lower'), _loop_vars=_loop_vars))
                    yield '-child-action"\n              >'
                    template = environment.get_template('_res/svg_ico16_add_child.jinja', 'components/node/node_controls/index.jinja')
                    gen = template.root_render_func(template.new_context(context.get_all(), True, {'element_': l_1_element_}))
                    try:
                        for event in gen:
                            yield event
                    finally: gen.close()
                    yield '</a>\n            </li>\n            '
                yield '\n            <li>\n              <a\n                href="/actions/document/new_requirement?reference_mid='
                yield escape(environment.getattr((undefined(name='sdoc_entity') if l_0_sdoc_entity is missing else l_0_sdoc_entity), 'reserved_mid'))
                yield '&whereto=after&element_type='
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield '&context_document_mid='
                yield escape(environment.getattr(environment.getattr((undefined(name='view_object') if l_0_view_object is missing else l_0_view_object), 'document'), 'reserved_mid'))
                yield '"\n                data-turbo-action="replace"\n                data-turbo="true"\n                title="Add '
                yield escape(environment.getattr(l_1_element_, 'tag'))
                yield ' below"\n                data-testid="node-add-'
                yield escape(context.call(environment.getattr(l_1_element_, 'get_tag_lower'), _loop_vars=_loop_vars))
                yield '-below-action"\n              >'
                template = environment.get_template('_res/svg_ico16_add_below.jinja', 'components/node/node_controls/index.jinja')
                gen = template.root_render_func(template.new_context(context.get_all(), True, {'element_': l_1_element_}))
                try:
                    for event in gen:
                        yield event
                finally: gen.close()
                yield '</a>\n            </li>'
            l_1_element_ = missing
        yield '</menu>\n    </sdoc-menu-list>\n  </sdoc-menu>'
    yield '</sdoc-node-controls>'

blocks = {}
debug_info = '17=14&21=17&27=19&35=26&36=28&38=31&44=37&46=44&52=50&55=60&61=64&70=73&72=76&79=80&87=87&105=90&116=97&119=100&121=104&128=106&131=112&132=114&133=116&141=124&143=127&144=131&149=133&152=139&153=141&154=143&158=150&161=156&162=158&163=160&167=167&170=173&171=175&172=177&180=186&182=189&183=193&186=195&191=199&194=205&195=207&196=209&198=216&201=219&204=225&205=227&206=229&211=237&214=243&215=245&216=247'