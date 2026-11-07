import { useShop } from './api/shop'

const usd = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' })

export default function App() {
  const shop = useShop()

  if (shop.isPending) return <main><p>Loading the shop…</p></main>
  if (shop.isError) {
    console.error(shop.error)
    return <main><p role="alert">Couldn't reach Beehive. {shop.error.message}</p></main>
  }

  const { shop: info, products, customers, orders } = shop.data
  return (
    <main>
      <h1>{info.name}</h1>
      <p>{info.location} · returns by Redo · agents through Beehive</p>
      <ul>
        {products.map((product) => (
          <li key={product.id}>
            {product.name} — {usd.format(product.price)}
            {product.billing === 'monthly' ? '/mo' : ''}
          </li>
        ))}
      </ul>
      <p>{customers.length} customers · {orders.length} orders</p>
    </main>
  )
}
