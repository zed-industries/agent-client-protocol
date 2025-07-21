import { mdxAnnotations } from 'mdx-annotations'
import remarkFrontmatter from 'remark-frontmatter'
import remarkGfm from 'remark-gfm'
import remarkMdxFrontmatter from 'remark-mdx-frontmatter'

export const remarkPlugins = [
  remarkFrontmatter,
  [remarkMdxFrontmatter, { name: 'metadata' }],
  mdxAnnotations.remark,
  remarkGfm,
]
