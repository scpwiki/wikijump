<?php /* Smarty version 2.6.7, created on 2009-01-04 17:48:11
         compiled from /var/www/wikidot/templates/modules/search/SearchAllModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/search/SearchAllModule.tpl', 6, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/search/SearchAllModule.tpl', 7, false),array('function', 'pager', '/var/www/wikidot/templates/modules/search/SearchAllModule.tpl', 28, false),)), $this); ?>
<div class="search-box">

	<div class="query-area">
		<form action="dummy" id="search-form-all">
			<div>
				<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Search query<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>: 
				<input class="text" type="text" size="30" name="query" id="search-form-all-input" value="<?php echo ((is_array($_tmp=$this->_tpl_vars['query'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
"/>
				<input class="button" type="submit" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>search<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>"/>
			</div>
			<div style="font-size: 87%; margin-top:5px;">
				<input id="search-all-pf" class="radio" type="radio" name="area" value="pf" <?php if (! $this->_tpl_vars['area'] || $this->_tpl_vars['area'] == 'pf'): ?>checked="checked"<?php endif; ?>/><label for="search-all-pf"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>pages and forums<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></label>
				<input id="search-all-p" class="radio" type="radio" name="area" value="p" <?php if ($this->_tpl_vars['area'] == 'p'): ?>checked="checked"<?php endif; ?>/><label for="search-all-p"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>pages only<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></label>
				<input id="search-all-f" class="radio" type="radio" name="area" value="f" <?php if ($this->_tpl_vars['area'] == 'f'): ?>checked="checked"<?php endif; ?>/><label for="search-all-f"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>forums only<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></label>
			</div>
			
			
		</form>
	</div>
		
	
	<?php if ($this->_tpl_vars['message']): ?>
		<p><?php echo $this->_tpl_vars['message']; ?>
</p>
	<?php endif; ?>
	
	<?php ob_start(); ?>/search:all<?php if ($this->_tpl_vars['area']): ?>/a/<?php echo $this->_tpl_vars['area'];  endif; ?>/q/<?php echo $this->_tpl_vars['queryEncoded']; ?>
/p/%d<?php $this->_smarty_vars['capture']['destUrl'] = ob_get_contents(); ob_end_clean(); ?>
	<?php echo smarty_function_pager(array('url' => $this->_smarty_vars['capture']['destUrl'],'total' => $this->_tpl_vars['pagerData']['total_pages'],'known' => $this->_tpl_vars['pagerData']['known_pages'],'current' => $this->_tpl_vars['pagerData']['current_page']), $this);?>

	
	<div class="search-results">
		<?php if ($this->_tpl_vars['results']): ?>
			<?php if (count($_from = (array)$this->_tpl_vars['results'])):
    foreach ($_from as $this->_tpl_vars['result']):
?>
				<div class="item">
					<div class="title">
												<a href="<?php echo $this->_tpl_vars['result']['url']; ?>
"><?php echo $this->_tpl_vars['result']['headline_title']; ?>
</a>
					</div>
					<div class="preview">
						<?php echo $this->_tpl_vars['result']['headline_text']; ?>

					</div> 
					<div class="site">
						site: <a href="http://<?php echo ((is_array($_tmp=$this->_tpl_vars['result']['site']->getDomain())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['result']['site']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
					</div>
					<div class="url">
						<?php echo $this->_tpl_vars['result']['url']; ?>
					</div>
				</div>
			<?php endforeach; endif; unset($_from); ?>
		<?php else: ?>
			<?php if ($this->_tpl_vars['query'] && $this->_tpl_vars['query'] != ''): ?>
				<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Sorry, no results found for your query.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
			<?php endif; ?>
		<?php endif; ?>
	</div>

	<?php if ($this->_tpl_vars['countResults'] > 7): ?>
		<?php echo smarty_function_pager(array('url' => $this->_smarty_vars['capture']['destUrl'],'total' => $this->_tpl_vars['pagerData']['total_pages'],'known' => $this->_tpl_vars['pagerData']['known_pages'],'current' => $this->_tpl_vars['pagerData']['current_page']), $this);?>

	
	<?php endif; ?>

</div>