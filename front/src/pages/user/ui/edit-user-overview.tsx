import { User } from '../../../api/api.interface';

type Props = {
  realm: string,
  user?: User
  currentView: string
}

export default function EditUserOverview(props: Props) {
  return (
    <p>{props.currentView}</p>
  )
}