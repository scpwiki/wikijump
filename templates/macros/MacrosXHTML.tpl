{**
* Renders the whole <head>...</head> content.
*}
{defmacro name="OzoneXHTMLHead"}
	<head>
		<title>{$page->getTitle()}</title>
	
	
    {foreach from=$page->getMetas() key=key item=content}
    	<meta name="{$key}" content="{$content}"/>
    {/foreach}
    {foreach from=$page->getHttpEquivs() key=key item=content}
    	<meta http-equiv="{$key}" content="{$content}"/>
    {/foreach}
  
    
    {foreach from=$page->getJavaScripts() item=js}
    	<script type="text/javascript" src="{$ui->javaScript($js)}"></script>
    {/foreach}
    
    {foreach from=$page->getLinks() item=lin}
    	<link rel="{$lin.rel}" href="{$lin.href}" 
    		{if $lin.type} type="{$lin.type}"{/if}
    		{if $lin.title} title="{$lin.title}"{/if}
    	/>
    {/foreach}
    
    <style type="text/css">
    {foreach from=$page->getStyleSheets() item=css}
    	@import url({$ui->style($css)});
  	{/foreach}  
  	{foreach from=$page->getStyleRaw() item=css}
  		{$css}
   {/foreach}
    </style>
    {if $page->hasJavaScriptRaw()}
    <script type="text/javascript">
		//  <![CDATA[
    	{foreach from=$page->getJavaScriptRaw() item=jsr}
    		{$jsr}
    	{/foreach}
    	// ]]>
		</script>
    {/if}
	{if $page->hasHeadRaw()}
	    {foreach from=$page->getHeadRaw() item=hr}
                {$hr}
            {/foreach}
    {/if}

    </head>
{/defmacro}

{**
* Renders <body> element attributes.
*}
{defmacro name="OzoneXHTMLBodyProperties"}
	{foreach from=$page->getBodyProperties() key=property item=value}
		{$property}="{$value}"
	{/foreach}
{/defmacro}